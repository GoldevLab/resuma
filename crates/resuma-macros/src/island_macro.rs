//! `#[island]` — optional boundary for heavy lazy JS or `load = "visible"`.
//!
//! Default resumability comes from [`component`](crate::component_macro). Use islands when
//! you need a separate chunk, visibility-gated load, or dev HMR via `/_resuma/island/:instance`.

use proc_macro2::TokenStream;
use quote::quote;
use syn::parse::{Parse, ParseStream};
use syn::{parse2, ItemFn, LitStr, Token};

struct IslandArgs {
    load: Option<LitStr>,
}

impl Parse for IslandArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut load = None;
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            if key == "load" {
                load = Some(input.parse()?);
            } else {
                return Err(syn::Error::new(key.span(), "unknown island attribute"));
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(Self { load })
    }
}

pub fn expand(args: TokenStream, input: TokenStream) -> TokenStream {
    let island_args = syn::parse2::<IslandArgs>(args).unwrap_or(IslandArgs { load: None });
    let load_policy = island_args
        .load
        .as_ref()
        .map(|s| s.value())
        .unwrap_or_else(|| "eager".to_string());

    let func: ItemFn = match parse2(input) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error(),
    };

    let name = func.sig.ident.clone();
    let name_str = name.to_string();
    let vis = func.vis.clone();
    let sig = func.sig.clone();
    let block = func.block.clone();

    quote! {
        #vis #sig {
            let __island_id = ::resuma::__private::current_context()
                .map(|c| c.next_signal_id().0)
                .unwrap_or(0);
            ::resuma::__private::wrap_in_island(#name_str, __island_id, || #block, #load_policy)
        }
    }
}
