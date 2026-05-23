//! `#[component]` — turns an `fn` into a Resuma component.
//!
//! For a function:
//!
//! ```ignore
//! #[component]
//! fn Counter(start: i32) -> View { ... }
//! ```
//!
//! we generate:
//!
//! 1. A `CounterProps` builder struct exposing fluent setters used by the
//!    `view!` macro when invoking `<Counter start={5} />`.
//! 2. A unit struct `Counter` implementing `Component`.
//! 3. A `fn Counter(props) -> View` that re-exports the original body.

use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{parse2, FnArg, ItemFn, Pat, ReturnType, Type};

pub fn expand(_args: TokenStream, input: TokenStream) -> TokenStream {
    let func: ItemFn = match parse2(input) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error(),
    };

    let name = func.sig.ident.clone();
    let vis = &func.vis;
    let body = &func.block;
    let return_ty = match &func.sig.output {
        ReturnType::Default => quote!(::resuma::__private::View),
        ReturnType::Type(_, ty) => quote!(#ty),
    };

    // Collect props: (name, type)
    let mut prop_fields = Vec::new();
    let mut prop_setters = Vec::new();
    let mut prop_destructure = Vec::new();

    for arg in &func.sig.inputs {
        let FnArg::Typed(pt) = arg else {
            return syn::Error::new(Span::call_site(), "components cannot have a self argument")
                .to_compile_error();
        };
        let Pat::Ident(pi) = &*pt.pat else {
            return syn::Error::new(
                Span::call_site(),
                "component arguments must be plain identifiers",
            )
            .to_compile_error();
        };
        let ident = pi.ident.clone();
        let ty: &Type = &pt.ty;
        prop_fields.push(quote! { pub #ident: #ty });
        let setter_name = ident.clone();
        prop_setters.push(quote! {
            pub fn #setter_name(mut self, value: #ty) -> Self {
                self.#ident = value;
                self
            }
        });
        prop_destructure.push(quote! { let #ident = props.#ident; });
    }

    let props_ident = format_ident!("{}Props", name);

    let expanded = quote! {
        #[allow(non_camel_case_types)]
        #vis struct #name;

        #[derive(Default, Clone)]
        #[allow(non_snake_case)]
        #vis struct #props_ident {
            #(#prop_fields,)*
            /// Children injected by the `view!` macro for `<Component>{...}</Component>`.
            #[doc(hidden)]
            pub __resuma_slotted: ::std::vec::Vec<::resuma::__private::SlottedChild>,
        }

        impl #props_ident {
            #(#prop_setters)*

            #[doc(hidden)]
            pub fn __resuma_slotted(mut self, c: ::std::vec::Vec<::resuma::__private::SlottedChild>) -> Self {
                self.__resuma_slotted = c;
                self
            }
        }

        impl ::resuma::__private::Component for #name {
            type Props = #props_ident;

            fn name() -> &'static str { stringify!(#name) }

            fn render(props: Self::Props) -> #return_ty {
                #(#prop_destructure)*
                let _slot_guard = ::resuma::__private::push_slots(props.__resuma_slotted);
                #body
            }
        }
    };

    expanded
}
