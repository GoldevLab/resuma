//! `debounce!([signals…], ms, move || { … })` — client debounced reaction.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{Expr, ExprClosure, Ident, Token};

struct DebounceInput {
    signals: Vec<Ident>,
    ms: Expr,
    closure: ExprClosure,
}

impl Parse for DebounceInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::bracketed!(content in input);
        let signals: Vec<Ident> = content
            .parse_terminated(Ident::parse, Token![,])?
            .into_iter()
            .collect();
        input.parse::<Token![,]>()?;
        let ms: Expr = input.parse()?;
        input.parse::<Token![,]>()?;
        let closure: ExprClosure = input.parse()?;
        Ok(Self {
            signals,
            ms,
            closure,
        })
    }
}

pub fn expand(input: TokenStream) -> TokenStream {
    let parsed = match syn::parse2::<DebounceInput>(input) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };

    let js = match crate::rs2js::translate_computed(&parsed.closure) {
        Ok(t) => t.js,
        Err(e) => {
            return syn::Error::new(
                parsed.closure.span(),
                crate::rs2js::translation_help("debounced effect", &e),
            )
            .to_compile_error();
        }
    };

    let ms = &parsed.ms;
    let closure = &parsed.closure;
    let primary = parsed
        .signals
        .first()
        .cloned()
        .unwrap_or_else(|| syn::Ident::new("_", proc_macro2::Span::call_site()));

    let mut capture_pairs = Vec::new();
    let mut clone_lets = Vec::new();
    for name in &parsed.signals {
        capture_pairs.push(quote! {
            (::std::string::ToString::to_string(stringify!(#name)), #name.id())
        });
        clone_lets.push(quote! { let #name = ::std::clone::Clone::clone(&#name); });
    }

    quote! {
        {
            let __captures = ::std::collections::BTreeMap::from([#(#capture_pairs),*]);
            {
                #(#clone_lets)*
                ::resuma::__private::use_effect(#closure);
            }
            ::resuma::__private::register_debounce_effect(
                &#primary,
                #ms,
                __captures,
                &#js,
            );
        }
    }
}
