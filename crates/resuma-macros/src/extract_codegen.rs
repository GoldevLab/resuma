//! Shared typed-extractor codegen for `#[load]`, `#[submit]`, and `#[server]`.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{FnArg, Pat, PatType, Type};

pub struct ExtractedParam {
    pub binding: TokenStream,
    pub call_expr: TokenStream,
}

/// Build `let` bindings and call arguments for typed FlowRequest extractors.
pub fn extract_flow_params(inputs: &[FnArg]) -> syn::Result<Vec<ExtractedParam>> {
    let mut out = Vec::new();
    for arg in inputs {
        let FnArg::Typed(PatType { pat, ty, .. }) = arg else {
            continue;
        };
        let Pat::Ident(pat_ident) = &**pat else {
            return Err(syn::Error::new_spanned(
                pat,
                "only simple identifier parameters are supported",
            ));
        };
        let ident = pat_ident.ident.clone();
        let (binding, call_expr) = extractor_binding(&ident, ty)?;
        out.push(ExtractedParam { binding, call_expr });
    }
    Ok(out)
}

fn extractor_binding(ident: &syn::Ident, ty: &Type) -> syn::Result<(TokenStream, TokenStream)> {
    if is_type(ty, "FlowRequest") {
        return Ok((quote! {}, quote!(req.clone())));
    }
    if is_ref_flow_request(ty) {
        return Ok((quote! {}, quote!(&req)));
    }
    if is_extractor(ty, "Path") {
        let inner = extractor_inner(ty);
        return Ok((
            quote! {
                let #ident = ::resuma::flow::Path::<#inner>::from_request(&req)?;
            },
            quote!(#ident.0),
        ));
    }
    if is_extractor(ty, "Query") {
        let inner = extractor_inner(ty);
        return Ok((
            quote! {
                let #ident = ::resuma::flow::Query::<#inner>::from_request(&req)?;
            },
            quote!(#ident.0),
        ));
    }
    Ok((quote! {}, quote!(#ident)))
}

fn is_ref_flow_request(ty: &Type) -> bool {
    let Type::Reference(r) = ty else {
        return false;
    };
    is_type(&r.elem, "FlowRequest")
}

fn is_type(ty: &Type, name: &str) -> bool {
    let Type::Path(tp) = ty else {
        return false;
    };
    tp.path
        .segments
        .last()
        .is_some_and(|s| s.ident == name)
}

fn is_extractor(ty: &Type, name: &str) -> bool {
    let Type::Path(tp) = ty else {
        return false;
    };
    let Some(seg) = tp.path.segments.last() else {
        return false;
    };
    if seg.ident != name {
        return false;
    }
    matches!(seg.arguments, syn::PathArguments::AngleBracketed(_))
}

fn extractor_inner(ty: &Type) -> Type {
    let Type::Path(tp) = ty else {
        return syn::parse_quote!(());
    };
    let Some(seg) = tp.path.segments.last() else {
        return syn::parse_quote!(());
    };
    if let syn::PathArguments::AngleBracketed(args) = &seg.arguments {
        if let Some(syn::GenericArgument::Type(t)) = args.args.first() {
            return t.clone();
        }
    }
    syn::parse_quote!(())
}
