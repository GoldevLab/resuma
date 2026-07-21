//! `#[upload]` — registers a named multipart handler at `POST /_resuma/upload/{name}`.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse::Parse, parse::ParseStream, parse2, FnArg, ItemFn, LitInt, LitStr, Pat, ReturnType,
    Token, Type,
};

struct UploadAttrs {
    max_bytes: Option<u64>,
    /// Exact mime strings, e.g. mime = "image/png,image/jpeg"
    mime: Option<String>,
}

impl Parse for UploadAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut max_bytes = None;
        let mut mime = None;
        while !input.is_empty() {
            let key: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;
            if key == "max_bytes" {
                let lit: LitInt = input.parse()?;
                max_bytes = Some(lit.base10_parse()?);
            } else if key == "mime" {
                let lit: LitStr = input.parse()?;
                mime = Some(lit.value());
            } else {
                return Err(syn::Error::new(
                    key.span(),
                    "unknown #[upload] attribute; use max_bytes = N and/or mime = \"a/b,c/d\"",
                ));
            }
            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }
        Ok(UploadAttrs { max_bytes, mime })
    }
}

fn return_type_is_result(output: &ReturnType) -> bool {
    match output {
        ReturnType::Type(_, ty) => match ty.as_ref() {
            Type::Path(p) => p
                .path
                .segments
                .last()
                .map(|s| s.ident == "Result")
                .unwrap_or(false),
            _ => false,
        },
        _ => false,
    }
}

pub fn expand(args: TokenStream, input: TokenStream) -> TokenStream {
    let attrs: UploadAttrs = if args.is_empty() {
        UploadAttrs {
            max_bytes: None,
            mime: None,
        }
    } else {
        match parse2(args) {
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
    let vis = &func.vis;
    let block = &func.block;
    let asyncness = &func.sig.asyncness;
    let output = &func.sig.output;
    let inputs = &func.sig.inputs;

    if asyncness.is_none() {
        return syn::Error::new(Span::call_site(), "#[upload] functions must be async")
            .to_compile_error();
    }

    let mut file_ident = None;
    for a in inputs {
        if let FnArg::Typed(pt) = a {
            if let Pat::Ident(pi) = &*pt.pat {
                file_ident = Some(pi.ident.clone());
            }
        }
    }
    let Some(file_ident) = file_ident else {
        return syn::Error::new(
            Span::call_site(),
            "#[upload] functions take one argument: `file: UploadedFile`",
        )
        .to_compile_error();
    };
    if inputs.len() != 1 {
        return syn::Error::new(
            Span::call_site(),
            "#[upload] functions take exactly one argument: `file: UploadedFile`",
        )
        .to_compile_error();
    }

    let max_bytes = attrs.max_bytes.unwrap_or(8 * 1024 * 1024);
    let mimes: Vec<String> = attrs
        .mime
        .as_deref()
        .unwrap_or("")
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let mime_lits = mimes.iter().map(|m| quote!(#m.to_string()));

    let returns_result = return_type_is_result(output);
    let serialize = if returns_result {
        quote! {
            match #name(#file_ident).await {
                Ok(v) => ::resuma::__private::serde_json::to_value(&v).map_err(|e| {
                    ::resuma::__private::ResumaError::Validation(format!(
                        "upload `{}` return encode failed: {}",
                        #name_str, e
                    ))
                }),
                Err(e) => Err(e),
            }
        }
    } else {
        quote! {
            ::resuma::__private::serde_json::to_value(&#name(#file_ident).await).map_err(|e| {
                ::resuma::__private::ResumaError::Validation(format!(
                    "upload `{}` return encode failed: {}",
                    #name_str, e
                ))
            })
        }
    };

    let trampoline = format_ident!("__resuma_upload_trampoline_{}", name);
    let registry_ctor = format_ident!("__resuma_upload_register_{}", name);

    quote! {
        #vis async fn #name ( #inputs ) #output #block

        #[doc(hidden)]
        fn #trampoline(
            file: ::resuma::exec::UploadedFile,
        ) -> ::std::pin::Pin<::std::boxed::Box<
            dyn ::std::future::Future<
                    Output = ::resuma::__private::Result<::resuma::__private::serde_json::Value>,
                > + Send,
        >> {
            ::std::boxed::Box::pin(async move {
                let #file_ident = file;
                #serialize
            })
        }

        #[doc(hidden)]
        #[::resuma::__private::ctor::ctor(unsafe, crate_path = ::resuma::__private::ctor)]
        fn #registry_ctor() {
            ::resuma::exec::register_upload(
                #name_str,
                ::resuma::exec::UploadMeta {
                    max_bytes: #max_bytes as usize,
                    mime: vec![#(#mime_lits),*],
                },
                #trampoline,
            );
        }
    }
}
