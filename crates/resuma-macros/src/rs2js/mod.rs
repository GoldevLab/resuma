//! Rust → JavaScript translator for Resuma's resumable event handlers.
//!
//! This is **not** a general purpose Rust-to-JS compiler. It accepts a small,
//! well-defined subset that is enough to express realistic event handlers
//! and ensures every supported construct has a clean JS counterpart that the
//! tiny client runtime understands.
//!
//! ## Supported subset
//!
//! * Closures: `|_| ...`, `move |ev| ...`, `|ev: MouseEvent| ...`
//! * Literals: integers, floats, booleans, strings (`"..."`).
//! * Operators: `+ - * / %  ==  !=  <  >  <=  >=  &&  || ! += -= *= /=`.
//! * Method calls on signals:
//!   * `s.get()`         → `state.s_NN.value`
//!   * `s.peek()`        → `state.s_NN.value`
//!   * `s.set(v)`        → `state.s_NN.set(v)`
//!   * `s.update(|c| ..)`→ `state.s_NN.update((c) => ..)`
//! * Macros: `format!("...{}", x)` → JS template literal.
//! * Method calls on strings/numbers that map 1:1 (`.len()` → `.length`,
//!   `.push_str(s)` → `+= s`, `.to_string()` → `String(...)`, etc.).
//! * Calls to server actions: `actions::name(args)` →
//!   `await __resuma.action('name', args)`.
//! * Calls to JS bridge: `js::bridge::name(args)` → `name(args)`.
//! * `if`/`else if`/`else`, blocks, semicolon statements.
//! * Variable bindings: `let x = ...;`.
//!
//! Anything outside this subset returns a `Rs2JsError` so the macro can
//! produce a friendly compile-time diagnostic pointing back at the original
//! Rust source span.

use std::collections::BTreeSet;

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{
    BinOp, Expr, ExprAssign, ExprBinary, ExprBlock, ExprCall, ExprClosure, ExprIf, ExprLit,
    ExprMacro, ExprMethodCall, ExprParen, ExprPath, ExprUnary, Lit, Local, Pat, Stmt, UnOp,
};

mod error;
pub use error::{translation_help, Rs2JsError};

/// Outcome of translating a single closure / expression.
#[derive(Debug, Clone)]
pub struct Translation {
    /// JavaScript source code.
    pub js: String,
    /// Identifiers (signal names) referenced from the host scope.
    pub captures: BTreeSet<String>,
    /// Server actions referenced (`actions::foo`).
    pub actions: BTreeSet<String>,
}

/// Translate a zero-arg reactive closure body for client effect replay.
pub fn translate_computed(closure: &ExprClosure) -> Result<Translation, Rs2JsError> {
    let mut t = Translator::default();
    t.locals.push(BTreeSet::new());
    for input in &closure.inputs {
        let (name, _) = pat_to_param(input)?;
        t.locals.last_mut().unwrap().insert(name);
    }
    let body_is_block = matches!(&*closure.body, Expr::Block(_));
    let body = t.expr(&closure.body)?;
    let body = if body_is_block {
        body
    } else {
        format!("return {};", body)
    };
    t.locals.pop();
    Ok(Translation {
        js: format!("(state, __resuma) => {{ {body} }}", body = body),
        captures: t.captures,
        actions: t.actions,
    })
}

/// Convenience entry point: translate a closure expression into a JS arrow
/// function. The closure must be a literal (e.g. `move |ev| ...`).
pub fn translate_handler(closure: &ExprClosure) -> Result<Translation, Rs2JsError> {
    let mut t = Translator::default();
    let body = t.translate_closure(closure, true)?;
    Ok(Translation {
        js: body,
        captures: t.captures,
        actions: t.actions,
    })
}

/// Translate an arbitrary expression — used by the `js!{}` escape hatch and
/// by the `view!` macro for inline reactive interpolation `{ count + 1 }`.
pub fn translate_expr(expr: &Expr) -> Result<Translation, Rs2JsError> {
    let mut t = Translator::default();
    let body = t.expr(expr)?;
    Ok(Translation {
        js: body,
        captures: t.captures,
        actions: t.actions,
    })
}

#[derive(Default)]
struct Translator {
    captures: BTreeSet<String>,
    actions: BTreeSet<String>,
    /// Locals that shadow captures inside the closure body.
    locals: Vec<BTreeSet<String>>,
}

impl Translator {
    fn translate_closure(&mut self, c: &ExprClosure, is_outer: bool) -> Result<String, Rs2JsError> {
        let mut params = Vec::new();
        self.locals.push(BTreeSet::new());

        for input in &c.inputs {
            let (name, _ty) = pat_to_param(input)?;
            let js_name = if is_outer && matches!(name.as_str(), "state" | "__resuma") {
                format!("_{name}")
            } else {
                name.clone()
            };
            self.locals.last_mut().unwrap().insert(name);
            params.push(js_name);
        }

        // Block bodies handle their own implicit `return` via `stmts`; any
        // other body is a single expression and must be wrapped with one so
        // helpers like `Signal.update(c => c + 1)` actually get the new value.
        let body_is_block = matches!(&*c.body, Expr::Block(_));
        let body = self.expr(&c.body)?;
        let body = if body_is_block {
            body
        } else {
            format!("return {};", body)
        };

        self.locals.pop();

        if is_outer {
            // Outer (handler) closures get the runtime calling convention
            // `(event, state, __resuma) => …`. The first user-declared param
            // (typically `_`) is mapped to `event`. We register `state` and
            // `__resuma` so they don't collide with user names — but they're
            // already provided by `Translator::path` via the `state.*` and
            // `actions::*` translation rules.
            let event_param = params.first().cloned().unwrap_or_else(|| "_".to_string());
            // Normalise leading `_` so the linter doesn't complain about
            // unused `event` if the handler ignores it (the runtime always
            // passes one).
            let event_alias = if event_param == "_" {
                "_event".to_string()
            } else {
                event_param
            };
            Ok(format!(
                "async ({event}, state, __resuma) => {{ {body} }}",
                event = event_alias,
                body = body,
            ))
        } else {
            Ok(format!("({}) => {{ {} }}", params.join(", "), body))
        }
    }

    fn expr(&mut self, e: &Expr) -> Result<String, Rs2JsError> {
        match e {
            Expr::Lit(ExprLit { lit, .. }) => self.lit(lit),

            Expr::Path(ExprPath { path, .. }) => self.path(path),

            Expr::Paren(ExprParen { expr, .. }) => Ok(format!("({})", self.expr(expr)?)),

            Expr::Unary(ExprUnary { op, expr, .. }) => {
                let inner = self.expr(expr)?;
                let op = match op {
                    UnOp::Not(_) => "!",
                    UnOp::Neg(_) => "-",
                    UnOp::Deref(_) => "",
                    _ => return Err(Rs2JsError::unsupported("unary op", e.span())),
                };
                Ok(format!("{}{}", op, inner))
            }

            Expr::Binary(ExprBinary {
                left, op, right, ..
            }) => {
                // syn 2 models compound assignments (`count += 1`) as
                // `Expr::Binary` with an `*Assign` `BinOp`. When the target is a
                // captured signal, `state.count` is a `SignalCell` (an object),
                // so a raw `state.count += 1` coerces it to `NaN` and destroys
                // the reference. Route signal targets through `.update()`.
                if let Some(arith) = compound_assign_arith(*op) {
                    let r = self.expr(right)?;
                    if let Some(name) = self.capture_lhs(left) {
                        return Ok(format!(
                            "state.{name}.update((__v) => (__v {arith} {r}))",
                            name = name,
                            arith = arith,
                            r = r
                        ));
                    }
                    let l = self.expr(left)?;
                    let op = bin_op_to_js(*op)
                        .ok_or_else(|| Rs2JsError::unsupported("binary op", e.span()))?;
                    return Ok(format!("({} {} {})", l, op, r));
                }
                let l = self.expr(left)?;
                let r = self.expr(right)?;
                let op = bin_op_to_js(*op)
                    .ok_or_else(|| Rs2JsError::unsupported("binary op", e.span()))?;
                Ok(format!("({} {} {})", l, op, r))
            }

            Expr::Assign(ExprAssign { left, right, .. }) => {
                let r = self.expr(right)?;
                if let Some(name) = self.capture_lhs(left) {
                    Ok(format!("state.{}.set({})", name, r))
                } else {
                    let l = self.expr(left)?;
                    Ok(format!("({} = {})", l, r))
                }
            }

            Expr::MethodCall(call) => self.method_call(call),

            Expr::Call(call) => self.call(call),

            Expr::Macro(ExprMacro { mac, .. }) => self.macro_call(mac),

            Expr::If(if_expr) => self.if_expr(if_expr),

            Expr::Block(ExprBlock { block, .. }) => {
                let stmts = self.stmts(&block.stmts)?;
                Ok(format!("(() => {{ {} }})()", stmts))
            }

            Expr::Closure(c) => self.translate_closure(c, false),

            Expr::Field(f) => {
                let base = self.expr(&f.base)?;
                match &f.member {
                    // Tuples/tuple-structs map to JS arrays, so `x.0` must use
                    // bracket indexing (`x[0]`) — `x.0` is invalid JS.
                    syn::Member::Named(id) => Ok(format!("{}.{}", base, id)),
                    syn::Member::Unnamed(idx) => Ok(format!("{}[{}]", base, idx.index)),
                }
            }

            Expr::Tuple(t) => {
                let items: Result<Vec<_>, _> = t.elems.iter().map(|e| self.expr(e)).collect();
                Ok(format!("[{}]", items?.join(", ")))
            }

            Expr::Array(a) => {
                let items: Result<Vec<_>, _> = a.elems.iter().map(|e| self.expr(e)).collect();
                Ok(format!("[{}]", items?.join(", ")))
            }

            Expr::Reference(r) => self.expr(&r.expr),

            Expr::Await(a) => Ok(format!("await {}", self.expr(&a.base)?)),

            other => Err(Rs2JsError::unsupported(&format!("{:?}", other), e.span())),
        }
    }

    fn stmts(&mut self, stmts: &[Stmt]) -> Result<String, Rs2JsError> {
        let mut out = Vec::with_capacity(stmts.len());
        for (i, s) in stmts.iter().enumerate() {
            let last = i == stmts.len() - 1;
            match s {
                Stmt::Local(Local { pat, init, .. }) => {
                    let (name, _ty) = pat_to_param(pat)?;
                    if let Some(scope) = self.locals.last_mut() {
                        scope.insert(name.clone());
                    }
                    let value = if let Some(init) = init {
                        self.expr(&init.expr)?
                    } else {
                        "undefined".into()
                    };
                    out.push(format!("let {} = {};", name, value));
                }
                Stmt::Expr(e, semi) => {
                    let js = self.expr(e)?;
                    if last && semi.is_none() {
                        out.push(format!("return {};", js));
                    } else {
                        out.push(format!("{};", js));
                    }
                }
                Stmt::Item(_) => {
                    return Err(Rs2JsError::unsupported("item statement", Span::call_site()))
                }
                Stmt::Macro(m) => {
                    let js = self.macro_call(&m.mac)?;
                    out.push(format!("{};", js));
                }
            }
        }
        Ok(out.join(" "))
    }

    fn lit(&self, lit: &Lit) -> Result<String, Rs2JsError> {
        match lit {
            Lit::Int(i) => Ok(i.base10_digits().to_string()),
            Lit::Float(f) => Ok(f.base10_digits().to_string()),
            Lit::Bool(b) => Ok(b.value.to_string()),
            Lit::Str(s) => Ok(format!("\"{}\"", escape_js_string(&s.value()))),
            Lit::Char(c) => Ok(format!("\"{}\"", escape_js_string(&c.value().to_string()))),
            other => Err(Rs2JsError::unsupported(
                &format!("literal: {:?}", other),
                Span::call_site(),
            )),
        }
    }

    /// If `left` is a simple captured signal identifier, register it as a
    /// capture and return its name. Assignments (`x = 1`, `x += 1`) must record
    /// the target signal so the runtime exposes it in the handler's `state`.
    fn capture_lhs(&mut self, left: &Expr) -> Option<String> {
        if let Expr::Path(ExprPath { path, .. }) = left {
            let segments: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
            if let [name] = segments.as_slice() {
                if !self.is_local(name) {
                    self.captures.insert(name.clone());
                    return Some(name.clone());
                }
            }
        }
        None
    }

    fn path(&mut self, path: &syn::Path) -> Result<String, Rs2JsError> {
        // Detect actions::foo / js::bridge::foo / locals / captures.
        let segments: Vec<String> = path.segments.iter().map(|s| s.ident.to_string()).collect();
        match segments.as_slice() {
            // Booleans and unit-like values that double as paths.
            [s] if s == "true" || s == "false" => Ok(s.clone()),

            [name] => {
                if self.is_local(name) {
                    Ok(name.clone())
                } else {
                    self.captures.insert(name.clone());
                    Ok(format!("state.{}", name))
                }
            }

            [ns, name] if ns == "actions" => {
                self.actions.insert(name.clone());
                Ok(format!("__resuma_action_{}", name))
            }

            [a, b, name] if a == "js" && b == "bridge" => Ok(name.clone()),

            other => Err(Rs2JsError::unsupported(
                &format!("path {:?}", other),
                path.span(),
            )),
        }
    }

    fn method_call(&mut self, call: &ExprMethodCall) -> Result<String, Rs2JsError> {
        let receiver = self.expr(&call.receiver)?;
        let method = call.method.to_string();

        // `signal.update(|c| *c += 1)` — in Rust the closure receives `&mut T`
        // and mutates in place (returning `()`). The JS `SignalCell.update`
        // contract instead expects the callback to *return* the next value, so
        // we translate the closure to return its (reassigned) parameter. This
        // keeps block-bodied closures (`|c| { *c += 1; }`) working, which would
        // otherwise return `undefined` and silently drop the update.
        if method == "update" {
            if let Some(Expr::Closure(closure)) = call.args.first() {
                if call.args.len() == 1 {
                    let cb = self.translate_update_closure(closure)?;
                    return Ok(format!("{}.update({})", receiver, cb));
                }
            }
        }

        let args: Result<Vec<_>, _> = call.args.iter().map(|a| self.expr(a)).collect();
        let args = args?;

        let js = match method.as_str() {
            // Signal API.
            "get" | "peek" | "value" => format!("{}.value", receiver),
            "set" => format!("{}.set({})", receiver, args.join(", ")),
            "update" => format!("{}.update({})", receiver, args.join(", ")),

            // Common Rust → JS sugar.
            "to_string" => format!("String({})", receiver),
            "len" => format!("{}.length", receiver),
            "is_empty" => format!("({}.length === 0)", receiver),
            "push" => format!("{}.push({})", receiver, args.join(", ")),
            "push_str" => format!("({} += {})", receiver, args.join(", ")),
            "pop" => format!("{}.pop()", receiver),
            "clone" => receiver,
            "as_str" => receiver,
            "into" => receiver,
            "iter" | "into_iter" | "iter_mut" => receiver,
            "map" => format!("{}.map({})", receiver, args.join(", ")),
            "filter" => format!("{}.filter({})", receiver, args.join(", ")),
            "collect" => receiver,
            "trim" => format!("{}.trim()", receiver),
            "to_lowercase" => format!("{}.toLowerCase()", receiver),
            "to_uppercase" => format!("{}.toUpperCase()", receiver),
            "contains" => format!("{}.includes({})", receiver, args.join(", ")),
            "starts_with" => format!("{}.startsWith({})", receiver, args.join(", ")),
            "ends_with" => format!("{}.endsWith({})", receiver, args.join(", ")),

            other => {
                return Err(Rs2JsError::unsupported(
                    &format!("method `.{}()`", other),
                    call.span(),
                ))
            }
        };
        Ok(js)
    }

    fn call(&mut self, call: &ExprCall) -> Result<String, Rs2JsError> {
        let func = self.expr(&call.func)?;
        let args: Result<Vec<_>, _> = call.args.iter().map(|a| self.expr(a)).collect();
        let args = args?;
        // actions::foo(arg) → await __resuma.action('foo', arg)
        if let Some(name) = func.strip_prefix("__resuma_action_") {
            return Ok(format!(
                "(await __resuma.action('{}', [{}]))",
                name,
                args.join(", ")
            ));
        }
        Ok(format!("{}({})", func, args.join(", ")))
    }

    fn macro_call(&mut self, mac: &syn::Macro) -> Result<String, Rs2JsError> {
        let name = mac
            .path
            .segments
            .last()
            .map(|s| s.ident.to_string())
            .unwrap_or_default();

        match name.as_str() {
            "format" => self.format_macro(mac),

            "vec" => {
                let tokens = mac.tokens.clone();
                let parsed = syn::parse::Parser::parse2(
                    syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated,
                    tokens,
                )
                .map_err(|e| Rs2JsError::unsupported(&format!("vec!: {}", e), mac.span()))?;
                let items: Result<Vec<_>, _> = parsed.iter().map(|e| self.expr(e)).collect();
                Ok(format!("[{}]", items?.join(", ")))
            }

            "println" | "print" | "eprintln" | "eprint" | "dbg" => {
                // Translate the format string like `format!` so `{}` holes are
                // interpolated instead of leaking raw Rust tokens into JS.
                let template = self.format_macro(mac)?;
                Ok(format!("console.log({})", template))
            }

            other => Err(Rs2JsError::unsupported(
                &format!("macro `{}!`", other),
                mac.span(),
            )),
        }
    }

    /// Translate a `format!`-style macro into a JS template literal, mapping
    /// `{}` holes to `${arg}` and escaping backticks / `$` / newlines / `</`.
    fn format_macro(&mut self, mac: &syn::Macro) -> Result<String, Rs2JsError> {
        let tokens = mac.tokens.clone();
        let parsed = syn::parse::Parser::parse2(
            syn::punctuated::Punctuated::<Expr, syn::Token![,]>::parse_terminated,
            tokens,
        )
        .map_err(|e| Rs2JsError::unsupported(&format!("format!: {}", e), mac.span()))?;
        let mut iter = parsed.into_iter();
        let fmt_lit = iter
            .next()
            .ok_or_else(|| Rs2JsError::unsupported("empty format!", mac.span()))?;
        let fmt = if let Expr::Lit(ExprLit {
            lit: Lit::Str(s), ..
        }) = &fmt_lit
        {
            s.value()
        } else {
            return Err(Rs2JsError::unsupported("format! needs literal", mac.span()));
        };

        let mut args = Vec::new();
        for a in iter {
            args.push(self.expr(&a)?);
        }

        let mut out = String::from("`");
        let mut arg_iter = args.into_iter();
        let mut chars = fmt.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '{' if chars.peek() == Some(&'{') => {
                    // `{{` is an escaped literal brace in Rust format strings.
                    chars.next();
                    out.push('{');
                }
                '}' if chars.peek() == Some(&'}') => {
                    chars.next();
                    out.push('}');
                }
                '{' => {
                    // Consume an optional format spec up to the closing `}`
                    // (e.g. `{}`, `{:?}`, `{:.2}`) — JS has no equivalent so we
                    // just interpolate the argument's default stringification.
                    for nc in chars.by_ref() {
                        if nc == '}' {
                            break;
                        }
                    }
                    if let Some(a) = arg_iter.next() {
                        out.push_str(&format!("${{{}}}", a));
                    }
                }
                '`' => out.push_str("\\`"),
                '$' => out.push_str("\\$"),
                '\\' => out.push_str("\\\\"),
                '<' => out.push_str("\\x3C"),
                '\r' => out.push_str("\\r"),
                c => out.push(c),
            }
        }
        out.push('`');
        Ok(out)
    }

    fn if_expr(&mut self, if_expr: &ExprIf) -> Result<String, Rs2JsError> {
        let inner = self.if_stmts(if_expr)?;
        Ok(format!("(() => {{ {} }})()", inner))
    }

    /// Emit `if (...) {...} else {...}` where **both** branches `return` their
    /// value. Previously the `else` branch was translated as a bare expression,
    /// so its value was computed and silently discarded (`if x { return a } else
    /// { b }` never returned `b`). Handles `else if` chains recursively.
    fn if_stmts(&mut self, if_expr: &ExprIf) -> Result<String, Rs2JsError> {
        let cond = self.expr(&if_expr.cond)?;
        let then = self.stmts(&if_expr.then_branch.stmts)?;
        let else_part = match &if_expr.else_branch {
            Some((_, else_b)) => match &**else_b {
                Expr::Block(b) => format!(" else {{ {} }}", self.stmts(&b.block.stmts)?),
                Expr::If(nested) => format!(" else {}", self.if_stmts(nested)?),
                other => format!(" else {{ return {}; }}", self.expr(other)?),
            },
            None => String::new(),
        };
        Ok(format!("if ({}) {{ {} }}{}", cond, then, else_part))
    }

    /// Translate a closure passed to `Signal::update`. The Rust closure mutates
    /// its `&mut T` param and returns `()`; the emitted JS arrow reassigns the
    /// param and returns it so `SignalCell.update` receives the next value.
    fn translate_update_closure(&mut self, c: &ExprClosure) -> Result<String, Rs2JsError> {
        self.locals.push(BTreeSet::new());

        let param = match c.inputs.first() {
            Some(input) => {
                let (name, _ty) = pat_to_param(input)?;
                self.locals.last_mut().unwrap().insert(name.clone());
                name
            }
            None => "_v".to_string(),
        };

        let body = match &*c.body {
            Expr::Block(ExprBlock { block, .. }) => self.stmts(&block.stmts)?,
            other => format!("{};", self.expr(other)?),
        };

        self.locals.pop();

        // Append `return <param>` so the (reassigned) value flows back out.
        Ok(format!(
            "({param}) => {{ {body} return {param}; }}",
            param = param,
            body = body
        ))
    }

    fn is_local(&self, name: &str) -> bool {
        self.locals.iter().rev().any(|s| s.contains(name))
    }
}

/// Escape a Rust string value for embedding inside a double-quoted JS string.
///
/// Beyond `\` and `"`, this handles newlines/control characters, the Unicode
/// line separators U+2028/U+2029 (which are literal line breaks in JS strings),
/// and neutralises `</script` / `<!--` so the generated JS can't break out of a
/// `<script>` block when inlined into HTML.
fn escape_js_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\\\"),
            '"' => out.push_str("\\\""),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\u{2028}' => out.push_str("\\u2028"),
            '\u{2029}' => out.push_str("\\u2029"),
            '<' => out.push_str("\\x3C"),
            other if (other as u32) < 0x20 => {
                out.push_str(&format!("\\u{:04x}", other as u32));
            }
            other => out.push(other),
        }
    }
    out
}

fn pat_to_param(p: &Pat) -> Result<(String, Option<String>), Rs2JsError> {
    match p {
        Pat::Ident(i) => Ok((i.ident.to_string(), None)),
        Pat::Wild(_) => Ok(("_".into(), None)),
        Pat::Type(t) => pat_to_param(&t.pat),
        other => Err(Rs2JsError::unsupported(
            &format!("pattern {:?}", other),
            Span::call_site(),
        )),
    }
}

/// Underlying arithmetic/bitwise operator for a compound-assignment `BinOp`
/// (`+=` → `+`). Returns `None` for non-assignment operators.
fn compound_assign_arith(op: BinOp) -> Option<&'static str> {
    Some(match op {
        BinOp::AddAssign(_) => "+",
        BinOp::SubAssign(_) => "-",
        BinOp::MulAssign(_) => "*",
        BinOp::DivAssign(_) => "/",
        BinOp::RemAssign(_) => "%",
        BinOp::BitAndAssign(_) => "&",
        BinOp::BitOrAssign(_) => "|",
        BinOp::BitXorAssign(_) => "^",
        BinOp::ShlAssign(_) => "<<",
        BinOp::ShrAssign(_) => ">>",
        _ => return None,
    })
}

fn bin_op_to_js(op: BinOp) -> Option<&'static str> {
    Some(match op {
        BinOp::Add(_) => "+",
        BinOp::Sub(_) => "-",
        BinOp::Mul(_) => "*",
        BinOp::Div(_) => "/",
        BinOp::Rem(_) => "%",
        BinOp::Eq(_) => "===",
        BinOp::Ne(_) => "!==",
        BinOp::Lt(_) => "<",
        BinOp::Le(_) => "<=",
        BinOp::Gt(_) => ">",
        BinOp::Ge(_) => ">=",
        BinOp::And(_) => "&&",
        BinOp::Or(_) => "||",
        BinOp::BitAnd(_) => "&",
        BinOp::BitOr(_) => "|",
        BinOp::BitXor(_) => "^",
        BinOp::Shl(_) => "<<",
        BinOp::Shr(_) => ">>",
        // syn 2 represents compound assignments as Expr::Binary with these
        // BinOp variants.
        BinOp::AddAssign(_) => "+=",
        BinOp::SubAssign(_) => "-=",
        BinOp::MulAssign(_) => "*=",
        BinOp::DivAssign(_) => "/=",
        BinOp::RemAssign(_) => "%=",
        BinOp::BitAndAssign(_) => "&=",
        BinOp::BitOrAssign(_) => "|=",
        BinOp::BitXorAssign(_) => "^=",
        BinOp::ShlAssign(_) => "<<=",
        BinOp::ShrAssign(_) => ">>=",
        _ => return None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn expr_js(src: &str) -> String {
        let e: Expr = syn::parse_str(src).expect("parse expr");
        translate_expr(&e).expect("translate").js
    }

    fn handler_js(src: &str) -> String {
        let c: ExprClosure = syn::parse_str(src).expect("parse closure");
        translate_handler(&c).expect("translate").js
    }

    #[test]
    fn tuple_index_uses_bracket_notation() {
        // `x.0` is invalid JS; must become `x[0]`.
        assert_eq!(expr_js("pair.0"), "state.pair[0]");
        assert_eq!(expr_js("pair.1"), "state.pair[1]");
    }

    #[test]
    fn if_else_returns_both_branches() {
        let js = expr_js("if flag { 1 } else { 2 }");
        // Both branches must `return` — the else value must not be discarded.
        assert!(js.contains("return 1;"), "then branch returns: {js}");
        assert!(js.contains("return 2;"), "else branch returns: {js}");
    }

    #[test]
    fn else_if_chain_returns_values() {
        let js = expr_js("if a { 1 } else if b { 2 } else { 3 }");
        assert!(js.contains("return 1;"));
        assert!(js.contains("return 2;"));
        assert!(js.contains("return 3;"));
        assert!(js.contains("else if"), "chained else-if preserved: {js}");
    }

    #[test]
    fn format_macro_handles_debug_and_precision_specs() {
        // `{:?}` / `{:.2}` specifiers must not break interpolation.
        let js = expr_js(r#"format!("val={:?} pi={:.2}", x, y)"#);
        assert!(js.contains("${state.x}"), "debug spec interpolated: {js}");
        assert!(
            js.contains("${state.y}"),
            "precision spec interpolated: {js}"
        );
    }

    #[test]
    fn format_macro_escapes_script_close() {
        let js = expr_js(r#"format!("</script>{}", x)"#);
        assert!(!js.contains("</script"), "must neutralize </script: {js}");
        assert!(js.contains("\\x3C"), "angle bracket escaped: {js}");
    }

    #[test]
    fn format_literal_braces() {
        let js = expr_js(r#"format!("{{literal}} {}", x)"#);
        assert!(js.contains("{literal}"), "escaped braces preserved: {js}");
        assert!(js.contains("${state.x}"));
    }

    #[test]
    fn string_literal_escapes_script_close_and_newlines() {
        let js = expr_js(r#""</script>\n""#);
        assert!(!js.contains("</script"), "escaped breakout: {js}");
        assert!(js.contains("\\n"), "newline escaped: {js}");
    }

    #[test]
    fn println_interpolates_instead_of_raw_tokens() {
        let js = expr_js(r#"println!("count={}", x)"#);
        assert!(js.starts_with("console.log("), "maps to console.log: {js}");
        assert!(js.contains("${state.x}"), "arg interpolated: {js}");
    }

    #[test]
    fn handler_signal_update_roundtrip() {
        let js = handler_js("move |_| count.update(|c| *c += 1)");
        assert!(js.contains("state.count.update"));
        assert!(js.contains("async (_event, state, __resuma)"));
    }

    #[test]
    fn update_expr_body_returns_mutated_param() {
        let js = expr_js("count.update(|c| *c += 1)");
        assert!(js.contains("state.count.update"), "got: {js}");
        assert!(js.contains("return c;"), "must return param: {js}");
        assert!(js.contains("c += 1"), "must reassign param: {js}");
    }

    #[test]
    fn update_block_body_returns_mutated_param() {
        // Block-bodied update closures previously returned `undefined` in JS,
        // silently dropping the update. Ensure the param is returned.
        let js = expr_js("count.update(|c| { *c += 2; })");
        assert!(js.contains("state.count.update"), "got: {js}");
        assert!(
            js.contains("return c;"),
            "block body must still return param: {js}"
        );
        assert!(js.contains("c += 2"), "got: {js}");
    }

    #[test]
    fn compound_assign_on_signal_uses_update() {
        let js = expr_js("count += 1");
        assert!(
            js.contains("state.count.update"),
            "compound assign on signal must use .update(): {js}"
        );
        assert!(
            !js.contains("state.count +="),
            "must not emit raw += on signal cell: {js}"
        );
        assert!(js.contains("+ 1"), "arithmetic preserved: {js}");
    }

    #[test]
    fn compound_assign_sub_on_signal() {
        let js = expr_js("total -= 5");
        assert!(js.contains("state.total.update"), "got: {js}");
        assert!(js.contains("- 5"), "got: {js}");
    }

    #[test]
    fn compound_assign_on_local_stays_raw() {
        let js = handler_js("move |n| { let x = n; x += 1 }");
        assert!(js.contains("x +="), "local vars keep raw += : {js}");
        assert!(
            !js.contains("state.x.update"),
            "must not treat let-binding as signal: {js}"
        );
    }
}
