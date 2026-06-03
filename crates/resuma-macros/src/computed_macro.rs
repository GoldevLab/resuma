//! `computed!([signals…], move || expr)` — rs2js-backed client computed.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::spanned::Spanned;
use syn::{ExprClosure, Ident, Token};

struct ComputedInput {
    signals: Vec<Ident>,
    closure: ExprClosure,
}

impl Parse for ComputedInput {
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
    let parsed = match syn::parse2::<ComputedInput>(input) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error(),
    };

    let js = match crate::rs2js::translate_computed(&parsed.closure) {
        Ok(t) => t.js,
        Err(e) => {
            return syn::Error::new(
                parsed.closure.span(),
                format!("computed! cannot translate closure: {e}"),
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
        clone_lets.push(quote! { let #name = ::std::clone::Clone::clone(&#name); });
    }

    let closure = &parsed.closure;
    quote! {
        {
            let __captures = ::std::collections::BTreeMap::from([#(#capture_pairs),*]);
            ::resuma::__private::use_computed_with_js(
                __captures,
                {
                    #(#clone_lets)*
                    #closure
                },
                #js,
            )
        }
    }
}
