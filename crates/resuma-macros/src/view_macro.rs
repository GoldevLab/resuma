//! `view!{}` — JSX-like template macro.
//!
//! Parsing strategy: hand-rolled recursive descent over `proc_macro2`
//! tokens. We chose this over `rstml` to keep the dependency footprint
//! tiny and to retain full control of how event handlers are translated
//! to JavaScript via `resuma-rs2js`.

use proc_macro2::{Delimiter, Literal, Span, TokenStream, TokenTree};
use quote::{quote, quote_spanned};
use std::iter::Peekable;
use std::vec::IntoIter;

use crate::rs2js;

#[derive(Debug)]
enum Node {
    Element {
        tag: String,
        attrs: Vec<Attr>,
        children: Vec<Node>,
        is_component: bool,
        self_closing: bool,
    },
    Fragment(Vec<Node>),
    Text(String),
    /// `{ expr }` interpolation — at SSR time we evaluate `expr` via
    /// `IntoView::into_view()`; for reactive expressions (`{count}`) we
    /// detect signal references in a follow-up pass.
    Expr(TokenStream),
}

#[derive(Debug)]
struct Attr {
    name: String,
    value: AttrVal,
}

#[derive(Debug)]
enum AttrVal {
    /// `attr="value"` — static string.
    StaticStr(String),
    /// `attr={ expr }` — dynamic Rust expression.
    Expr(TokenStream),
    /// Boolean attribute without `=value` (e.g. `disabled`).
    Bool,
}

pub fn expand(input: TokenStream) -> TokenStream {
    let tokens: Vec<TokenTree> = input.into_iter().collect();
    let mut iter = tokens.into_iter().peekable();

    let nodes = match parse_nodes(&mut iter, false) {
        Ok(n) => n,
        Err(err) => {
            return quote! { compile_error!(#err); };
        }
    };

    // A view! invocation that produces a single root returns it as `View`.
    // Multiple top-level children become a fragment.
    let body = if nodes.len() == 1 {
        emit_node(nodes.into_iter().next().unwrap())
    } else {
        let children = nodes.into_iter().map(emit_child);
        quote! {
            ::resuma::__private::View::fragment(vec![ #(#children),* ])
        }
    };

    quote! { { use ::resuma::__private::*; #body } }
}

// ---------- parser ----------

type TokenIter = Peekable<IntoIter<TokenTree>>;

fn parse_nodes(iter: &mut TokenIter, in_element: bool) -> Result<Vec<Node>, String> {
    let mut nodes = Vec::new();
    while let Some(tt) = iter.peek().cloned() {
        match tt {
            TokenTree::Punct(p) if p.as_char() == '<' => {
                // Could be `</` (closing tag) — let caller handle.
                if is_closing_tag(iter) {
                    return Ok(nodes);
                }
                nodes.push(parse_element(iter)?);
            }
            TokenTree::Group(g) if g.delimiter() == Delimiter::Brace => {
                iter.next();
                nodes.push(Node::Expr(g.stream()));
            }
            TokenTree::Literal(lit) => {
                iter.next();
                nodes.push(Node::Text(unquote_string(&lit)));
            }
            TokenTree::Ident(id) if !in_element => {
                iter.next();
                nodes.push(Node::Text(id.to_string()));
            }
            _ => {
                if in_element {
                    iter.next();
                } else {
                    break;
                }
            }
        }
    }
    Ok(nodes)
}

fn is_closing_tag(iter: &mut TokenIter) -> bool {
    let mut peeker = iter.clone();
    if let Some(TokenTree::Punct(p)) = peeker.next() {
        if p.as_char() == '<' {
            if let Some(TokenTree::Punct(p2)) = peeker.next() {
                return p2.as_char() == '/';
            }
        }
    }
    false
}

fn parse_element(iter: &mut TokenIter) -> Result<Node, String> {
    expect_punct(iter, '<')?;

    // Fragment: `<>...</>`
    if let Some(TokenTree::Punct(p)) = iter.peek() {
        if p.as_char() == '>' {
            iter.next();
            let children = parse_nodes(iter, true)?;
            // Expect `</>`
            expect_punct(iter, '<')?;
            expect_punct(iter, '/')?;
            expect_punct(iter, '>')?;
            return Ok(Node::Fragment(children));
        }
    }

    let tag = parse_ident_path(iter)?;
    let is_component = tag.chars().next().is_some_and(|c| c.is_uppercase());

    let mut attrs = Vec::new();
    loop {
        match iter.peek() {
            Some(TokenTree::Punct(p)) if p.as_char() == '/' => {
                iter.next();
                expect_punct(iter, '>')?;
                return Ok(Node::Element {
                    tag,
                    attrs,
                    children: vec![],
                    is_component,
                    self_closing: true,
                });
            }
            Some(TokenTree::Punct(p)) if p.as_char() == '>' => {
                iter.next();
                break;
            }
            Some(TokenTree::Ident(_)) => attrs.push(parse_attr(iter)?),
            Some(other) => {
                return Err(format!("unexpected token in opening tag: {}", other));
            }
            None => return Err("unterminated opening tag".into()),
        }
    }

    let children = parse_nodes(iter, true)?;

    // closing `</tag>`
    expect_punct(iter, '<')?;
    expect_punct(iter, '/')?;
    let close_tag = parse_ident_path(iter)?;
    if close_tag != tag {
        return Err(format!(
            "mismatched closing tag: expected </{}>, got </{}>",
            tag, close_tag
        ));
    }
    expect_punct(iter, '>')?;

    Ok(Node::Element {
        tag,
        attrs,
        children,
        is_component,
        self_closing: false,
    })
}

fn parse_attr(iter: &mut TokenIter) -> Result<Attr, String> {
    let name = match iter.next() {
        Some(TokenTree::Ident(id)) => id.to_string(),
        other => return Err(format!("expected attribute name, got {:?}", other)),
    };

    // Allow attributes like `data-foo` — but `-` is two tokens, so detect.
    let mut full_name = name;
    while let Some(TokenTree::Punct(p)) = iter.peek() {
        if p.as_char() == '-' {
            iter.next();
            if let Some(TokenTree::Ident(id)) = iter.next() {
                full_name.push('-');
                full_name.push_str(&id.to_string());
            } else {
                return Err("expected ident after `-`".into());
            }
        } else {
            break;
        }
    }
    // colon-namespaced attrs (`xlink:href`).
    if let Some(TokenTree::Punct(p)) = iter.peek() {
        if p.as_char() == ':' {
            iter.next();
            if let Some(TokenTree::Ident(id)) = iter.next() {
                full_name.push(':');
                full_name.push_str(&id.to_string());
            }
        }
    }

    let value = match iter.peek() {
        Some(TokenTree::Punct(p)) if p.as_char() == '=' => {
            iter.next();
            match iter.next() {
                Some(TokenTree::Literal(lit)) => AttrVal::StaticStr(unquote_string(&lit)),
                Some(TokenTree::Group(g)) if g.delimiter() == Delimiter::Brace => {
                    AttrVal::Expr(g.stream())
                }
                Some(TokenTree::Ident(id)) => AttrVal::Expr(quote!(#id)),
                other => return Err(format!("expected attribute value, got {:?}", other)),
            }
        }
        _ => AttrVal::Bool,
    };

    Ok(Attr {
        name: full_name,
        value,
    })
}

fn parse_ident_path(iter: &mut TokenIter) -> Result<String, String> {
    let mut out = String::new();
    if let Some(TokenTree::Ident(id)) = iter.next() {
        out.push_str(&id.to_string());
    } else {
        return Err("expected tag name".into());
    }
    // Allow `Foo::Bar` for components.
    loop {
        let mut peeker = iter.clone();
        if let (Some(TokenTree::Punct(p1)), Some(TokenTree::Punct(p2))) =
            (peeker.next(), peeker.next())
        {
            if p1.as_char() == ':' && p2.as_char() == ':' {
                iter.next();
                iter.next();
                if let Some(TokenTree::Ident(id)) = iter.next() {
                    out.push_str("::");
                    out.push_str(&id.to_string());
                    continue;
                }
            }
        }
        break;
    }
    Ok(out)
}

fn expect_punct(iter: &mut TokenIter, c: char) -> Result<(), String> {
    match iter.next() {
        Some(TokenTree::Punct(p)) if p.as_char() == c => Ok(()),
        other => Err(format!("expected `{}`, got {:?}", c, other)),
    }
}

fn unquote_string(lit: &Literal) -> String {
    let s = lit.to_string();
    syn::parse_str::<syn::LitStr>(&s)
        .map(|lit| lit.value())
        .unwrap_or(s)
}

// ---------- emitter ----------

fn emit_node(node: Node) -> TokenStream {
    match node {
        Node::Text(t) => quote! { ::resuma::__private::View::Text(#t.to_string()) },
        // Take a reference so signals captured elsewhere are not moved.
        Node::Expr(ts) => quote! { ::resuma::__private::IntoView::into_view(&(#ts)) },
        Node::Fragment(children) => {
            let cs = children.into_iter().map(emit_child);
            quote! { ::resuma::__private::View::fragment(vec![ #(#cs),* ]) }
        }
        Node::Element {
            tag,
            attrs,
            children,
            is_component,
            self_closing,
        } => {
            if tag == "Slot" {
                emit_slot(attrs)
            } else if tag == "Form" {
                emit_form(attrs, children)
            } else if tag == "NavLink" {
                emit_nav_link(attrs, children)
            } else if tag == "Show" {
                emit_show(attrs, children)
            } else if is_component {
                emit_component(tag, attrs, children)
            } else {
                emit_html_element(tag, attrs, children, self_closing)
            }
        }
    }
}

fn emit_child(node: Node) -> TokenStream {
    match node {
        Node::Text(t) => quote! { ::resuma::__private::Child::Text(#t.to_string()) },
        other => {
            let v = emit_node(other);
            quote! { ::resuma::__private::Child::View(#v) }
        }
    }
}

fn emit_slot(attrs: Vec<Attr>) -> TokenStream {
    let mut name: Option<String> = None;
    for a in attrs {
        if a.name.as_str() == "name" {
            if let AttrVal::StaticStr(s) = a.value {
                name = Some(s);
            } else if let AttrVal::Expr(ts) = a.value {
                return quote! { ::resuma::__private::resolve_slot(Some({ #ts }.to_string().as_str())) };
            }
        }
    }
    match name {
        Some(n) => {
            let lit = n;
            quote! { ::resuma::__private::resolve_slot(Some(#lit)) }
        }
        None => quote! { ::resuma::__private::resolve_slot(None) },
    }
}

fn emit_html_element(
    tag: String,
    attrs: Vec<Attr>,
    children: Vec<Node>,
    _self_closing: bool,
) -> TokenStream {
    let attr_pushes = attrs.into_iter().map(emit_attr);
    let child_pushes = children.into_iter().map(emit_child);
    quote! {
        ::resuma::__private::View::element(#tag)
            #( .attr_runtime(#attr_pushes) )*
            .children(vec![ #(#child_pushes),* ])
            .build()
    }
}

fn emit_component(tag: String, attrs: Vec<Attr>, children: Vec<Node>) -> TokenStream {
    let component_path: TokenStream = tag.parse().unwrap_or_else(|_| quote!(MissingComponent));

    let setters = attrs.into_iter().map(|a| {
        let name = syn::Ident::new(&a.name, Span::call_site());
        match a.value {
            AttrVal::StaticStr(s) => quote! { .#name(#s) },
            AttrVal::Expr(ts) => quote! { .#name({ #ts }) },
            AttrVal::Bool => quote! { .#name(true) },
        }
    });

    let child_pushes = children.into_iter().map(emit_slotted_child);

    quote! {
        ::resuma::__private::render_component::<#component_path>(
            <#component_path as ::resuma::__private::Component>::Props::default()
                #(#setters)*
                .__resuma_slotted(vec![ #(#child_pushes),* ])
        )
    }
}

fn emit_nav_link(attrs: Vec<Attr>, children: Vec<Node>) -> TokenStream {
    let mut href: Option<TokenStream> = None;
    let mut active_class = quote! { "active" };
    let mut class = quote! { "" };

    for a in attrs {
        match a.name.as_str() {
            "href" => {
                href = Some(match a.value {
                    AttrVal::StaticStr(s) => quote!(#s),
                    AttrVal::Expr(ts) => quote!({ #ts }),
                    AttrVal::Bool => quote! { "" },
                });
            }
            "activeClass" | "active_class" => {
                active_class = match a.value {
                    AttrVal::StaticStr(s) => quote!(#s),
                    AttrVal::Expr(ts) => quote!({ #ts }),
                    AttrVal::Bool => quote! { "active" },
                };
            }
            "class" => {
                class = match a.value {
                    AttrVal::StaticStr(s) => quote!(#s),
                    AttrVal::Expr(ts) => quote!({ #ts }),
                    AttrVal::Bool => quote! { "" },
                };
            }
            _ => {}
        }
    }

    let href = href.unwrap_or_else(|| quote! { "" });
    let child_pushes = children.into_iter().map(emit_child);

    quote! {
        {
            let __path = ::resuma::current_request()
                .map(|r| r.path)
                .unwrap_or_else(|| "/".to_string());
            ::resuma::__private::nav_link(
                #href,
                &__path,
                #active_class,
                #class,
                vec![ #(#child_pushes),* ],
            )
        }
    }
}

fn emit_show(attrs: Vec<Attr>, children: Vec<Node>) -> TokenStream {
    let mut when = quote! { true };
    let mut fallback = quote! { None::<::resuma::__private::View> };

    for a in attrs {
        match a.name.as_str() {
            "when" => {
                when = match a.value {
                    AttrVal::StaticStr(s) => quote!(#s == "true"),
                    AttrVal::Expr(ts) => quote!({ #ts }),
                    AttrVal::Bool => quote!(true),
                };
            }
            "fallback" => {
                fallback = match a.value {
                    AttrVal::Expr(ts) => quote! { Some({ #ts }) },
                    AttrVal::StaticStr(s) => {
                        quote! { Some(::resuma::__private::View::text(#s)) }
                    }
                    AttrVal::Bool => quote! { Some(::resuma::__private::View::empty()) },
                };
            }
            _ => {}
        }
    }

    let child_pushes = children.into_iter().map(emit_child);

    quote! {
        ::resuma::__private::show(
            #when,
            vec![ #(#child_pushes),* ],
            #fallback,
        )
    }
}

fn emit_form(attrs: Vec<Attr>, children: Vec<Node>) -> TokenStream {
    let mut submit_name: Option<TokenStream> = None;
    let mut extra_attrs = Vec::new();

    for a in attrs {
        if a.name == "submit" {
            submit_name = Some(match a.value {
                AttrVal::StaticStr(s) => quote!(#s),
                AttrVal::Expr(ts) => {
                    if let Ok(path) = syn::parse2::<syn::Path>(ts.clone()) {
                        if let Some(segment) = path.segments.last() {
                            let ident = segment.ident.clone();
                            quote!(stringify!(#ident))
                        } else {
                            quote!({ #ts }.to_string())
                        }
                    } else {
                        quote!({ #ts }.to_string())
                    }
                }
                AttrVal::Bool => {
                    return compile_err("Form submit handler must be a function name");
                }
            });
        } else {
            extra_attrs.push(a);
        }
    }

    let submit = submit_name.unwrap_or_else(|| {
        quote! { compile_error!("Form requires submit={handler}"); "" }
    });

    let attr_pushes = extra_attrs.into_iter().map(emit_attr);
    let child_pushes = children.into_iter().map(emit_child);

    quote! {
        ::resuma::__private::flow_form(
            #submit,
            vec![ #(#attr_pushes),* ],
            vec![ #(#child_pushes),* ],
        )
    }
}

fn emit_slotted_child(node: Node) -> TokenStream {
    match node {
        Node::Element {
            mut attrs,
            children,
            is_component,
            self_closing,
            tag,
        } => {
            let slot_name = take_slot_attr(&mut attrs);
            let child = if tag == "Slot" {
                emit_slot(attrs)
            } else if tag == "Form" {
                emit_form(attrs, children)
            } else if tag == "NavLink" {
                emit_nav_link(attrs, children)
            } else if tag == "Show" {
                emit_show(attrs, children)
            } else if is_component {
                emit_component(tag, attrs, children)
            } else {
                emit_html_element(tag, attrs, children, self_closing)
            };
            let slot_expr = match slot_name {
                Some(AttrVal::StaticStr(s)) => quote! { Some(#s.to_string()) },
                Some(AttrVal::Expr(ts)) => quote! { Some({ #ts }.to_string()) },
                Some(AttrVal::Bool) | None => quote! { None },
            };
            quote! {
                ::resuma::__private::SlottedChild {
                    slot: #slot_expr,
                    child: ::resuma::__private::Child::View(#child),
                }
            }
        }
        other => {
            let child = emit_child(other);
            quote! {
                ::resuma::__private::SlottedChild {
                    slot: None,
                    child: #child,
                }
            }
        }
    }
}

fn take_slot_attr(attrs: &mut Vec<Attr>) -> Option<AttrVal> {
    attrs
        .iter()
        .position(|a| a.name == "slot")
        .map(|idx| attrs.remove(idx).value)
}

fn emit_attr(attr: Attr) -> TokenStream {
    let name = attr.name.clone();
    let lower = name.to_lowercase();

    if lower == "preventdefault" {
        return emit_modifier_attr("preventDefault", attr.value);
    }
    if lower == "stoppropagation" {
        return emit_modifier_attr("stopPropagation", attr.value);
    }

    // Detect event handlers in any of the common spellings:
    //   * Solid-style `on:click=...`
    //   * React-style `onClick=...`   (lowercased to "onclick" for matching)
    //   * HTML-style  `onclick=...`
    let is_event = name.starts_with("on:") || lower.starts_with("on") && lower.len() > 2;

    if is_event {
        return emit_event_handler(name, attr.value);
    }

    match attr.value {
        AttrVal::StaticStr(s) => quote! {
            (#name.to_string(), ::resuma::__private::AttrValue::Static(#s.to_string()))
        },
        AttrVal::Bool => quote! {
            (#name.to_string(), ::resuma::__private::AttrValue::Bool(true))
        },
        AttrVal::Expr(ts) => quote! {
            (#name.to_string(), ::resuma::__private::resolve_attr_value({ #ts }))
        },
    }
}

fn emit_modifier_attr(kind: &str, value: AttrVal) -> TokenStream {
    let event = match value {
        AttrVal::StaticStr(s) => quote!(#s.to_string()),
        AttrVal::Bool => quote!("click".to_string()),
        AttrVal::Expr(ts) => quote!({ #ts }.to_string()),
    };
    match kind {
        "preventDefault" => quote! {
            ("preventDefault".to_string(), ::resuma::__private::AttrValue::PreventDefault(#event))
        },
        _ => quote! {
            ("stopPropagation".to_string(), ::resuma::__private::AttrValue::StopPropagation(#event))
        },
    }
}

fn emit_event_handler(attr_name: String, value: AttrVal) -> TokenStream {
    let event = attr_name
        .strip_prefix("on:")
        .map(|s| s.to_string())
        .unwrap_or_else(|| {
            let lower = attr_name.to_lowercase();
            lower.strip_prefix("on").unwrap_or(&lower).to_string()
        });

    let (js_source, captures, actions): (String, Vec<String>, Vec<String>) = match &value {
        AttrVal::Expr(ts) => {
            // `js!{...}` escape hatch: take the inner tokens verbatim as JS.
            if let Some(js) = extract_js_macro(ts) {
                let captures = scan_state_refs(&js);
                let body = format!("(async (event, state, __resuma) => {{ {} }})", js);
                (body, captures, vec![])
            } else {
                let parsed: Result<syn::Expr, _> = syn::parse2(ts.clone());
                match parsed {
                    Ok(syn::Expr::Closure(c)) => match rs2js::translate_handler(&c) {
                        Ok(t) => (
                            t.js,
                            t.captures.into_iter().collect(),
                            t.actions.into_iter().collect(),
                        ),
                        Err(e) => {
                            return compile_err_at(e.span, &handler_translation_help(&e.message))
                        }
                    },
                    Ok(other) => match rs2js::translate_expr(&other) {
                        Ok(t) => (
                            format!("async (_event, state, __resuma) => {{ {}; }}", t.js),
                            t.captures.into_iter().collect(),
                            t.actions.into_iter().collect(),
                        ),
                        Err(e) => {
                            return compile_err_at(e.span, &handler_translation_help(&e.message))
                        }
                    },
                    Err(e) => return compile_err(&format!("invalid handler expression: {}", e)),
                }
            }
        }
        AttrVal::StaticStr(s) => (s.clone(), vec![], vec![]),
        AttrVal::Bool => return compile_err("event handlers must have a value"),
    };

    // Stable symbol per (file, line, attr) position.
    let symbol = stable_symbol(&attr_name, &js_source);
    let chunk = "__page__".to_string();

    let captures_lits: Vec<TokenStream> = captures
        .iter()
        .map(|c| {
            let id = syn::Ident::new(c, Span::call_site());
            let name_lit = c.clone();
            quote! {
                ::resuma::__private::ResumeCapture::Signal {
                    name: #name_lit.to_string(),
                    id: #id.id(),
                }
            }
        })
        .collect();
    let action_lits: Vec<TokenStream> = actions.iter().map(|a| quote! { #a.to_string() }).collect();

    quote! {
        (
            #attr_name.to_string(),
            ::resuma::__private::register_handler(
                #event,
                #chunk,
                #symbol,
                #js_source,
                vec![ #(#captures_lits),* ],
                vec![ #(#action_lits),* ],
            )
        )
    }
}

/// Scan a JS source string for `state.<ident>` references and return the
/// unique idents in source order. Used to wire up captures for `js!{...}`
/// escape-hatch handlers — a low-cost way to know which signals must be
/// exposed to the runtime without forcing the user to declare them.
fn scan_state_refs(src: &str) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let bytes = src.as_bytes();
    let mut i = 0;
    while i + 6 <= bytes.len() {
        // Look for the substring "state." preceded by a non-ident byte.
        if &bytes[i..i + 6] == b"state." {
            let prev_ok = i == 0 || !is_ident_byte(bytes[i - 1]);
            if prev_ok {
                let mut j = i + 6;
                while j < bytes.len() && is_ident_byte(bytes[j]) {
                    j += 1;
                }
                if j > i + 6 {
                    let name = std::str::from_utf8(&bytes[i + 6..j])
                        .unwrap_or("")
                        .to_string();
                    if !out.contains(&name) {
                        out.push(name);
                    }
                }
                i = j;
                continue;
            }
        }
        i += 1;
    }
    out
}

fn is_ident_byte(b: u8) -> bool {
    b.is_ascii_alphanumeric() || b == b'_' || b == b'$'
}

/// Returns the inner tokens of a `js!{...}` invocation if the input is
/// exactly that macro call (otherwise `None`).
fn extract_js_macro(ts: &TokenStream) -> Option<String> {
    let tokens: Vec<TokenTree> = ts.clone().into_iter().collect();
    // Expect: `js` `!` `{...}` — with optional path leading to `js`.
    let mut iter = tokens.into_iter();
    let first = iter.next()?;
    let ident = match first {
        TokenTree::Ident(i) if i == "js" => i,
        _ => return None,
    };
    let _ = ident;
    let bang = iter.next()?;
    if let TokenTree::Punct(p) = &bang {
        if p.as_char() != '!' {
            return None;
        }
    } else {
        return None;
    }
    let group = iter.next()?;
    if let TokenTree::Group(g) = group {
        if g.delimiter() != Delimiter::Brace && g.delimiter() != Delimiter::Parenthesis {
            return None;
        }
        if iter.next().is_some() {
            return None;
        }
        return Some(g.stream().to_string());
    }
    None
}

fn stable_symbol(attr: &str, js: &str) -> String {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    let mut h = DefaultHasher::new();
    attr.hash(&mut h);
    js.hash(&mut h);
    format!("h_{:x}", h.finish())
}

fn compile_err(msg: &str) -> TokenStream {
    compile_err_at(Span::call_site(), msg)
}

fn compile_err_at(span: Span, msg: &str) -> TokenStream {
    let lit = Literal::string(msg);
    quote_spanned! {span=>
        (String::from(""), { compile_error!(#lit); ::resuma::__private::AttrValue::Bool(false) })
    }
}

fn handler_translation_help(message: &str) -> String {
    format!(
        "Resuma could not compile this event handler to browser JavaScript: {message}.\n\n\
         Supported handlers are intentionally small and resumable. Try one of these:\n\
         - update signals directly: onClick={{count.update(|c| *c += 1)}}\n\
         - use js! {{ ... }} for DOM/browser APIs\n\
         - move complex Rust or database work into a #[server] action"
    )
}
