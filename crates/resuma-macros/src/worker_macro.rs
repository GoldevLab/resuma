//! `#[worker]` — registers an async fn as a Resuma execution worker.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::Parse, parse::ParseStream, parse2, FnArg, ItemFn, LitStr, Pat, ReturnType, Token, Type,
};

struct WorkerAttrs {
    intent: LitStr,
    resources: Option<LitStr>,
}

impl Parse for WorkerAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut intent = None;
        let mut resources = None;
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            if key == "intent" {
                intent = Some(input.parse()?);
            } else if key == "resources" {
                resources = Some(input.parse()?);
            } else {
                return Err(syn::Error::new(key.span(), "unknown #[worker] attribute; use intent = \"...\" and optional resources = \"auto\" | \"extended\" | \"none\" | \"<secs>\""));
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(WorkerAttrs {
            intent: intent.ok_or_else(|| {
                syn::Error::new(Span::call_site(), "#[worker] requires intent = \"...\"")
            })?,
            resources,
        })
    }
}

pub fn expand(args: TokenStream, input: TokenStream) -> TokenStream {
    let attrs: WorkerAttrs = match parse2(args) {
        Ok(a) => a,
        Err(e) => return e.to_compile_error(),
    };

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
        return syn::Error::new(Span::call_site(), "#[worker] functions must be async")
            .to_compile_error();
    }

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

    let (call_idents, has_ctx) = split_ctx_arg(&arg_idents, &arg_types);
    if call_idents.len() != 1 {
        return syn::Error::new(
            Span::call_site(),
            "#[worker] functions take exactly one input argument (serde_json::Value) before optional WorkerContext",
        )
        .to_compile_error();
    }

    let input_ident = &call_idents[0];
    let intent_str = attrs.intent.value();
    let resources_expr = match &attrs.resources {
        Some(lit) => {
            let v = lit.value();
            match v.as_str() {
                "auto" => quote!(::resuma::exec::Resources::auto()),
                "extended" => quote!(::resuma::exec::Resources::extended()),
                "none" | "unlimited" => quote!(::resuma::exec::Resources::unlimited()),
                other => {
                    // Allow numeric strings: resources = "600" → 600s wall timeout.
                    if other.parse::<u64>().is_ok() {
                        let s = other.to_string();
                        quote!(::resuma::exec::Resources {
                            timeout: ::resuma::exec::ResourceLevel::Named(#s.into()),
                            ..::resuma::exec::Resources::auto()
                        })
                    } else {
                        return syn::Error::new(
                            lit.span(),
                            "resources must be \"auto\" | \"extended\" | \"none\" | \"<secs>\"",
                        )
                        .to_compile_error();
                    }
                }
            }
        }
        None => quote!(::resuma::exec::Resources::auto()),
    };

    let call = if has_ctx {
        quote!( #name( #input_ident, ctx ) )
    } else {
        quote!( #name( #input_ident ) )
    };

    let returns_result = return_type_is_result(output);
    let serialize_result = if returns_result {
        quote! {
            match #call.await {
                Ok(v) => ::resuma::__private::serde_json::to_value(&v).map_err(|e| {
                    ::resuma::__private::ResumaError::Validation(format!(
                        "worker `{}` return encode failed: {}",
                        #name_str, e
                    ))
                }),
                Err(e) => Err(e),
            }
        }
    } else {
        quote! {
            ::resuma::__private::serde_json::to_value(&#call.await).map_err(|e| {
                ::resuma::__private::ResumaError::Validation(format!(
                    "worker `{}` return encode failed: {}",
                    #name_str, e
                ))
            })
        }
    };

    let runner_name = format_ident!("__resuma_worker_run_{}", name);
    let trampoline_name = format_ident!("__resuma_worker_trampoline_{}", name);
    let registry_ctor = format_ident!("__resuma_worker_register_{}", name);

    quote! {
        #vis async fn #name ( #inputs ) #output #block

        #[doc(hidden)]
        async fn #runner_name(
            input: ::resuma::__private::serde_json::Value,
            ctx: ::resuma::exec::WorkerContext,
        ) -> ::resuma::__private::Result<::resuma::__private::serde_json::Value> {
            let #input_ident: _ = ::resuma::__private::serde_json::from_value(input).map_err(|e| {
                ::resuma::__private::ResumaError::Validation(format!(
                    "worker `{}` input decode failed: {}",
                    #name_str, e
                ))
            })?;
            #serialize_result
        }

        #[doc(hidden)]
        fn #trampoline_name(
            input: ::resuma::__private::serde_json::Value,
            ctx: ::resuma::exec::WorkerContext,
        ) -> ::std::pin::Pin<::std::boxed::Box<
            dyn ::std::future::Future<
                Output = ::resuma::__private::Result<::resuma::__private::serde_json::Value>,
            > + ::std::marker::Send,
        >> {
            ::std::boxed::Box::pin(#runner_name(input, ctx))
        }

        #[doc(hidden)]
        #[::resuma::__private::ctor::ctor(unsafe, crate_path = ::resuma::__private::ctor)]
        fn #registry_ctor() {
            ::resuma::exec::register_worker(
                #name_str,
                ::resuma::exec::WorkerMeta {
                    intent: #intent_str.into(),
                    resources: #resources_expr,
                },
                #trampoline_name,
            );
        }
    }
}

fn split_ctx_arg(idents: &[syn::Ident], types: &[Type]) -> (Vec<syn::Ident>, bool) {
    if idents.is_empty() {
        return (Vec::new(), false);
    }
    let last_ty = &types[types.len() - 1];
    if is_worker_context(last_ty) {
        (idents[..idents.len() - 1].to_vec(), true)
    } else {
        (idents.to_vec(), false)
    }
}

fn is_worker_context(ty: &Type) -> bool {
    match ty {
        Type::Path(p) => p
            .path
            .segments
            .last()
            .is_some_and(|s| s.ident == "WorkerContext"),
        _ => false,
    }
}

fn return_type_is_result(output: &ReturnType) -> bool {
    let ReturnType::Type(_, ty) = output else {
        return false;
    };
    let Type::Path(p) = &**ty else {
        return false;
    };
    p.path.segments.last().is_some_and(|s| s.ident == "Result")
}
