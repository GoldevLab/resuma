//! `#[derive(Store)]` — field getters/setters via a local extension trait.

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Fields};

pub fn derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let trait_name = syn::Ident::new(&format!("{name}Store"), name.span());

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return syn::Error::new_spanned(
                    &input,
                    "#[derive(Store)] requires a struct with named fields",
                )
                .to_compile_error()
                .into()
            }
        },
        _ => {
            return syn::Error::new_spanned(&input, "#[derive(Store)] supports structs only")
                .to_compile_error()
                .into()
        }
    };

    let mut getters = Vec::new();
    let mut setters = Vec::new();
    let mut trait_methods = Vec::new();

    for field in fields {
        let ident = field.ident.as_ref().expect("named field");
        let ty = &field.ty;
        let setter = syn::Ident::new(&format!("set_{ident}"), ident.span());

        trait_methods.push(quote! {
            fn #ident(&self) -> #ty;
            fn #setter(&self, value: #ty);
        });
        getters.push(quote! {
            fn #ident(&self) -> #ty {
                ::resuma::Store::get(self).#ident
            }
        });
        setters.push(quote! {
            fn #setter(&self, value: #ty) {
                ::resuma::Store::update(self, |s| s.#ident = value);
            }
        });
    }

    let expanded = quote! {
        pub trait #trait_name {
            #(#trait_methods)*
        }

        impl #trait_name for ::resuma::Store<#name> {
            #(#getters)*
            #(#setters)*
        }
    };

    expanded.into()
}
