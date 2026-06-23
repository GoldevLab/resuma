//! `#[middleware]` — registers request middleware for Resuma Flow.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse2, ItemFn, ReturnType};

pub fn expand(_args: TokenStream, input: TokenStream) -> TokenStream {
    let func: ItemFn = match parse2(input) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error(),
    };

    let name = func.sig.ident.clone();
    let vis = &func.vis;
    let inputs = &func.sig.inputs;
    let output = &func.sig.output;
    let block = &func.block;

    if func.sig.asyncness.is_none() {
        return syn::Error::new(Span::call_site(), "#[middleware] functions must be async")
            .to_compile_error();
    }

    let return_ty = match output {
        ReturnType::Type(_, ty) => ty.clone(),
        ReturnType::Default => {
            return syn::Error::new(
                Span::call_site(),
                "#[middleware] must return FlowRequest or Result",
            )
            .to_compile_error();
        }
    };

    let dispatcher = format_ident!("__resuma_middleware_dispatch_{}", name);
    let trampoline = format_ident!("__resuma_middleware_trampoline_{}", name);
    let registry = format_ident!("__resuma_middleware_register_{}", name);

    quote! {
        #vis async fn #name ( #inputs ) #output #block

        #[doc(hidden)]
        pub async fn #dispatcher(req: ::resuma::FlowRequest) -> #return_ty {
            #name(req).await
        }

        #[doc(hidden)]
        fn #trampoline(req: ::resuma::FlowRequest) -> ::std::pin::Pin<::std::boxed::Box<
            dyn ::std::future::Future<Output = #return_ty> + ::std::marker::Send,
        >> {
            ::std::boxed::Box::pin(#dispatcher(req))
        }

        #[doc(hidden)]
        #[::resuma::__private::ctor::ctor(unsafe, crate_path = ::resuma::__private::ctor)]
        fn #registry() {
            ::resuma::register_middleware(#trampoline);
        }
    }
}
