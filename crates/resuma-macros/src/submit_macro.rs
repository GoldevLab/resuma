//! `#[submit]` — registers a form submission handler for Resuma Flow.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse2, FnArg, ItemFn, Pat, ReturnType, Type};

use crate::extract_codegen::extract_flow_params;

pub fn expand(_args: TokenStream, input: TokenStream) -> TokenStream {
    let func: ItemFn = match parse2(input) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error(),
    };

    let name = func.sig.ident.clone();
    let name_str = name.to_string();
    let vis = &func.vis;
    let inputs = func.sig.inputs.clone();
    let output = &func.sig.output;
    let block = &func.block;

    if func.sig.asyncness.is_none() {
        return syn::Error::new(Span::call_site(), "#[submit] functions must be async")
            .to_compile_error();
    }

    let mut arg_idents = Vec::new();
    let mut rest_inputs = Vec::new();
    let mut first = true;
    for a in &inputs {
        if let FnArg::Typed(pt) = a {
            if let Pat::Ident(pi) = &*pt.pat {
                arg_idents.push(pi.ident.clone());
                if !first {
                    rest_inputs.push(a.clone());
                }
                first = false;
            }
        }
    }

    let rest_extracted = match extract_flow_params(&rest_inputs) {
        Ok(p) => p,
        Err(e) => return e.to_compile_error(),
    };
    let rest_bindings: Vec<TokenStream> = rest_extracted.iter().map(|p| p.binding.clone()).collect();
    let rest_call_args: Vec<TokenStream> = rest_extracted.iter().map(|p| p.call_expr.clone()).collect();

    let dispatcher = format_ident!("__resuma_submit_dispatch_{}", name);
    let trampoline = format_ident!("__resuma_submit_trampoline_{}", name);
    let registry = format_ident!("__resuma_submit_register_{}", name);

    let (data_ident, _rest_idents): (syn::Ident, Vec<syn::Ident>) = match arg_idents.split_first() {
        Some((first, rest)) => (first.clone(), rest.to_vec()),
        None => {
            return syn::Error::new(
                Span::call_site(),
                "#[submit] needs at least a data argument",
            )
            .to_compile_error();
        }
    };

    let call = if rest_call_args.is_empty() {
        quote!( #name( #data_ident ) )
    } else {
        quote!( #name( #data_ident, #(#rest_call_args),* ) )
    };

    let encode_result = if is_result_type(output) {
        quote! {
            ::resuma::encode_submit_result(#call.await)
        }
    } else {
        quote! {
            {
                let res = #call.await;
                ::resuma::__private::serde_json::to_value(&res).map_err(|e| {
                    ::resuma::__private::ResumaError::Validation(format!(
                        "Could not encode return value from submit `{}`: {}. If the return value is your own struct or enum, add #[data] above its definition.",
                        #name_str,
                        e,
                    ))
                })
            }
        }
    };

    quote! {
        #vis async fn #name ( #inputs ) #output #block

        #[doc(hidden)]
        pub async fn #dispatcher(
            data: ::resuma::__private::serde_json::Value,
            req: ::resuma::FlowRequest,
        ) -> ::resuma::__private::Result<::resuma::__private::serde_json::Value> {
            let #data_ident: _ = ::resuma::__private::serde_json::from_value(data.clone())
                .map_err(|e| ::resuma::__private::ResumaError::Validation(format!(
                    "Could not decode form data for submit `{}` into `{}`: {}. If `{}` is your own struct or enum, add #[data] above its definition.",
                    #name_str,
                    stringify!(#data_ident),
                    e,
                    stringify!(#data_ident),
                )))?;
            #(#rest_bindings)*
            #encode_result
        }

        #[doc(hidden)]
        fn #trampoline(
            data: ::resuma::__private::serde_json::Value,
            req: ::resuma::FlowRequest,
        ) -> ::std::pin::Pin<::std::boxed::Box<
            dyn ::std::future::Future<Output = ::resuma::__private::Result<::resuma::__private::serde_json::Value>> + ::std::marker::Send,
        >> {
            ::std::boxed::Box::pin(#dispatcher(data, req))
        }

        #[doc(hidden)]
        #[::resuma::__private::ctor::ctor]
        fn #registry() {
            ::resuma::register_submit(#name_str, #trampoline);
        }
    }
}

fn is_result_type(output: &ReturnType) -> bool {
    let ReturnType::Type(_, ty) = output else {
        return false;
    };
    let Type::Path(tp) = ty.as_ref() else {
        return false;
    };
    tp.path
        .segments
        .last()
        .map(|s| s.ident == "Result")
        .unwrap_or(false)
}
