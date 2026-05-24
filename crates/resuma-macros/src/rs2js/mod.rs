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
pub use error::Rs2JsError;

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
            self.locals.last_mut().unwrap().insert(name.clone());
            params.push(name);
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
                let l = self.expr(left)?;
                let r = self.expr(right)?;
                let op = bin_op_to_js(*op)
                    .ok_or_else(|| Rs2JsError::unsupported("binary op", e.span()))?;
                Ok(format!("({} {} {})", l, op, r))
            }

            Expr::Assign(ExprAssign { left, right, .. }) => {
                let l = self.expr(left)?;
                let r = self.expr(right)?;
                Ok(format!("({} = {})", l, r))
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
                let member = match &f.member {
                    syn::Member::Named(id) => id.to_string(),
                    syn::Member::Unnamed(idx) => idx.index.to_string(),
                };
                Ok(format!("{}.{}", base, member))
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
            Lit::Str(s) => Ok(format!(
                "\"{}\"",
                s.value().replace('\\', "\\\\").replace('"', "\\\"")
            )),
            Lit::Char(c) => Ok(format!("\"{}\"", c.value())),
            other => Err(Rs2JsError::unsupported(
                &format!("literal: {:?}", other),
                Span::call_site(),
            )),
        }
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
            "format" => {
                // format!("hello {}", x) → `hello ${state.x}`
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
                        '{' if chars.peek() == Some(&'}') => {
                            chars.next();
                            if let Some(a) = arg_iter.next() {
                                out.push_str(&format!("${{{}}}", a));
                            }
                        }
                        '`' => out.push_str("\\`"),
                        '$' => out.push_str("\\$"),
                        c => out.push(c),
                    }
                }
                out.push('`');
                Ok(out)
            }

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

            "println" | "dbg" | "eprintln" => {
                let tokens = mac.tokens.clone();
                Ok(format!("console.log({})", tokens))
            }

            other => Err(Rs2JsError::unsupported(
                &format!("macro `{}!`", other),
                mac.span(),
            )),
        }
    }

    fn if_expr(&mut self, if_expr: &ExprIf) -> Result<String, Rs2JsError> {
        let cond = self.expr(&if_expr.cond)?;
        let then = self.stmts(&if_expr.then_branch.stmts)?;
        let else_part = if let Some((_, else_b)) = &if_expr.else_branch {
            let e = self.expr(else_b)?;
            format!(" else {{ {} }}", e)
        } else {
            String::new()
        };
        Ok(format!(
            "(() => {{ if ({}) {{ {} }}{} }})()",
            cond, then, else_part
        ))
    }

    fn is_local(&self, name: &str) -> bool {
        self.locals.iter().rev().any(|s| s.contains(name))
    }
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
