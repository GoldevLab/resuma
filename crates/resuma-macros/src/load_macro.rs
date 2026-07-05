//! `#[load]` — registers an async data loader for Resuma Flow pages.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::parse::{Parse, ParseStream};
use syn::{parse2, ItemFn, LitStr, ReturnType};

use crate::extract_codegen::extract_flow_params;

struct LoadArgs {
    cache: Option<LitStr>,
    stream: bool,
}

impl Parse for LoadArgs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut cache = None;
        let mut stream = false;
        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            if ident == "stream" {
                stream = true;
            } else if ident == "cache" {
                input.parse::<syn::Token![=]>()?;
                cache = Some(input.parse()?);
            } else {
                return Err(input.error("supported attributes: cache, stream"));
            }
            if input.peek(syn::Token![,]) {
                input.parse::<syn::Token![,]>()?;
            }
        }
        Ok(LoadArgs { cache, stream })
    }
}

pub fn expand(args: TokenStream, input: TokenStream) -> TokenStream {
    let load_args = if args.is_empty() {
        LoadArgs {
            cache: None,
            stream: false,
        }
    } else {
        match syn::parse2::<LoadArgs>(args) {
            Ok(a) => a,
            Err(e) => return e.to_compile_error(),
        }
    };

    let func: ItemFn = match parse2(input) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error(),
    };

    let name = func.sig.ident.clone();
    let name_str = name.to_string();
    let use_fn = format_ident!("use_{}_load", name);
    let try_fn = format_ident!("try_{}_load", name);
    let stream_view_fn = format_ident!("{}_stream_view", name);
    let vis = &func.vis;
    let inputs = &func.sig.inputs;
    let output = &func.sig.output;
    let block = &func.block;

    if func.sig.asyncness.is_none() {
        return syn::Error::new(Span::call_site(), "#[load] functions must be async")
            .to_compile_error();
    }

    let return_ty = match output {
        ReturnType::Type(_, ty) => ty.clone(),
        ReturnType::Default => {
            return syn::Error::new(Span::call_site(), "#[load] must return a value")
                .to_compile_error();
        }
    };

    let dispatcher = format_ident!("__resuma_load_dispatch_{}", name);
    let trampoline = format_ident!("__resuma_load_trampoline_{}", name);
    let registry = format_ident!("__resuma_load_register_{}", name);
    let cache_registry = format_ident!("__resuma_load_cache_register_{}", name);
    let stream_registry = format_ident!("__resuma_load_stream_register_{}", name);
    let stream_chunk_fn = format_ident!("__resuma_stream_chunk_{}", name);
    let stream_chunk_registry = format_ident!("__resuma_stream_chunk_register_{}", name);

    let register_cache = match &load_args.cache {
        Some(lit) => quote! {
            #[doc(hidden)]
            #[::resuma::__private::ctor::ctor(unsafe, crate_path = ::resuma::__private::ctor)]
            fn #cache_registry() {
                ::resuma::register_loader_cache(#name_str, #lit);
            }
        },
        None => quote! {},
    };

    let stream_registration = if load_args.stream {
        quote! {
            #[doc(hidden)]
            #[::resuma::__private::ctor::ctor(unsafe, crate_path = ::resuma::__private::ctor)]
            fn #stream_registry() {
                ::resuma::register_stream_loader(#name_str);
            }

            #[doc(hidden)]
            fn #stream_chunk_fn(value: &::resuma::__private::serde_json::Value) -> ::resuma::View {
                match ::resuma::__private::serde_json::from_value::<#return_ty>(value.clone()) {
                    Ok(data) => #stream_view_fn(&data),
                    Err(_) => ::resuma::View::empty(),
                }
            }

            #[doc(hidden)]
            #[::resuma::__private::ctor::ctor(unsafe, crate_path = ::resuma::__private::ctor)]
            fn #stream_chunk_registry() {
                ::resuma::register_stream_chunk(#name_str, #stream_chunk_fn);
            }
        }
    } else {
        quote! {}
    };

    let use_accessor = if load_args.stream {
        quote! {
            #vis fn #use_fn() -> ::resuma::LoadValue<#return_ty> {
                ::resuma::try_use_load_value(#name_str)
            }

            /// Fallible accessor — never panics; pair with `error_boundary`.
            #vis fn #try_fn() -> ::std::result::Result<#return_ty, ::resuma::LoaderError> {
                ::resuma::try_use_load(#name_str)
            }
        }
    } else {
        quote! {
            #vis fn #use_fn() -> ::resuma::LoadValue<#return_ty> {
                match ::resuma::try_use_load(#name_str) {
                    Ok(v) => ::resuma::LoadValue::Ok(v),
                    Err(e) => ::resuma::LoadValue::Err(e),
                }
            }

            /// Fallible accessor — returns the loader error instead of panicking.
            #vis fn #try_fn() -> ::std::result::Result<#return_ty, ::resuma::LoaderError> {
                ::resuma::try_use_load(#name_str)
            }
        }
    };

    let extracted = match extract_flow_params(&inputs.iter().cloned().collect::<Vec<_>>()) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error(),
    };
    let call_args: Vec<TokenStream> = if extracted.is_empty() {
        vec![quote!(&req)]
    } else {
        extracted.iter().map(|p| p.call_expr.clone()).collect()
    };
    let param_bindings: Vec<TokenStream> = extracted.iter().map(|p| p.binding.clone()).collect();

    quote! {
        #vis async fn #name ( #inputs ) #output #block

        #[doc(hidden)]
        pub async fn #dispatcher(req: ::resuma::FlowRequest) -> ::resuma::__private::Result<::resuma::__private::serde_json::Value> {
            #(#param_bindings)*
            let res = #name( #(#call_args),* ).await;
            ::resuma::__private::serde_json::to_value(&res)
                .map_err(::resuma::__private::ResumaError::from)
        }

        #[doc(hidden)]
        fn #trampoline(req: ::resuma::FlowRequest) -> ::std::pin::Pin<::std::boxed::Box<
            dyn ::std::future::Future<Output = ::resuma::__private::Result<::resuma::__private::serde_json::Value>> + ::std::marker::Send,
        >> {
            ::std::boxed::Box::pin(#dispatcher(req))
        }

        #[doc(hidden)]
        #[::resuma::__private::ctor::ctor(unsafe, crate_path = ::resuma::__private::ctor)]
        fn #registry() {
            ::resuma::register_loader(#name_str, #trampoline);
        }

        #register_cache
        #stream_registration
        #use_accessor
    }
}
