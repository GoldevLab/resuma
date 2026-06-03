//! `use_effect!([signals…], move || { … })` — rs2js-backed client effect replay.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{ExprClosure, Ident, Token};

struct EffectInput {
    signals: Vec<Ident>,
    closure: ExprClosure,
}

impl Parse for EffectInput {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::bracketed!(content in input);
        let signals: Vec<Ident> = content
            .parse_terminated(Ident::parse, Token![,])?
            .into_iter()
            .collect();
        input.parse::<Token![,]>()?;
        let closure: ExprClosure = input.parse()?;
        Ok(Self { signals, closure })
    }
}

pub fn expand(input: TokenStream) -> TokenStream {
    let parsed = match syn::parse2::<EffectInput>(input) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };

    let js = match crate::rs2js::translate_computed(&parsed.closure) {
        Ok(t) => t.js,
        Err(e) => {
            return syn::Error::new(
                parsed.closure.span(),
                format!("use_effect! cannot translate closure: {e}"),
            )
            .to_compile_error();
        }
    };

    let mut capture_pairs = Vec::new();
    let mut clone_lets = Vec::new();
    for name in &parsed.signals {
        capture_pairs.push(quote! {
            (::std::string::ToString::to_string(stringify!(#name)), #name.id())
        });
        // Clone listed signals into the closure so the originals stay usable
        // (e.g. rendered) after the macro, and ids are read before the move.
        clone_lets.push(quote! { let #name = ::std::clone::Clone::clone(&#name); });
    }

    let closure = &parsed.closure;
    quote! {
        {
            let __captures = ::std::collections::BTreeMap::from([#(#capture_pairs),*]);
            let __eff = {
                #(#clone_lets)*
                ::resuma::__private::use_effect(#closure)
            };
            ::resuma::__private::attach_client_effect(
                &__eff,
                "effect",
                #js,
                __captures,
                None,
                None,
            );
            __eff
        }
    }
}
