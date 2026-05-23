//! `#[server]` — exposes an async fn as a server action.
//!
//! Generates:
//!  * a wrapper that registers the action in the global registry
//!  * a typed client stub callable from `view!` handlers as `actions::name(..)`

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse2, FnArg, ItemFn, Pat};

pub fn expand(_args: TokenStream, input: TokenStream) -> TokenStream {
    let func: ItemFn = match parse2(input) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error(),
    };

    let name = func.sig.ident.clone();
    let name_str = name.to_string();
    let vis = &func.vis;
    let block = &func.block;
    let asyncness = &func.sig.asyncness;
    let output = &func.sig.output;

    if asyncness.is_none() {
        return syn::Error::new(Span::call_site(), "#[server] functions must be async")
            .to_compile_error();
    }

    // Forward args verbatim. Each must be (Pat, Type).
    let inputs = func.sig.inputs.clone();

    let mut arg_idents = Vec::new();
    let mut arg_types = Vec::new();
    for a in &inputs {
        if let FnArg::Typed(pt) = a {
            if let Pat::Ident(pi) = &*pt.pat {
                arg_idents.push(pi.ident.clone());
                arg_types.push((*pt.ty).clone());
            }
        }
    }

    let dispatcher_name = format_ident!("__resuma_action_dispatch_{}", name);
    let registry_ctor   = format_ident!("__resuma_action_register_{}", name);
    let trampoline_name = format_ident!("__resuma_action_trampoline_{}", name);

    let json_extract = arg_idents.iter().enumerate().map(|(i, id)| {
        quote! {
            let #id: _ = match args.get(#i).cloned() {
                Some(v) => match ::resuma::__private::serde_json::from_value(v) {
                    Ok(v) => v,
                    Err(e) => return Err(::resuma::__private::ResumaError::Other(format!("bad arg `{}`: {}", stringify!(#id), e))),
                },
                None => return Err(::resuma::__private::ResumaError::Other(format!("missing arg `{}`", stringify!(#id)))),
            };
        }
    });

    quote! {
        #vis async fn #name ( #inputs ) #output #block

        #[doc(hidden)]
        pub async fn #dispatcher_name(
            args: ::std::vec::Vec<::resuma::__private::serde_json::Value>,
        ) -> ::resuma::__private::Result<::resuma::__private::serde_json::Value>
        {
            #(#json_extract)*
            let res = #name( #(#arg_idents),* ).await;
            ::resuma::__private::serde_json::to_value(&res)
                .map_err(::resuma::__private::ResumaError::from)
        }

        /// Trampoline whose explicit signature lets the compiler coerce the
        /// `Box::pin(dispatcher_future)` value into the boxed-future type
        /// expected by `resuma_server::ActionFn`.
        #[doc(hidden)]
        fn #trampoline_name (
            args: ::std::vec::Vec<::resuma::__private::serde_json::Value>,
        ) -> ::std::pin::Pin<::std::boxed::Box<
            dyn ::std::future::Future<
                Output = ::resuma::__private::Result<::resuma::__private::serde_json::Value>,
            > + ::std::marker::Send,
        >> {
            ::std::boxed::Box::pin(#dispatcher_name(args))
        }

        #[doc(hidden)]
        #[::resuma::__private::ctor::ctor]
        fn #registry_ctor() {
            ::resuma::__private::register_server_action(#name_str, #trampoline_name);
        }
    }
}
