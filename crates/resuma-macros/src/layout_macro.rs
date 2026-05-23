//! `#[layout]` — registers a layout component for a URL prefix.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, ItemFn, LitStr};

pub fn expand(args: TokenStream, input: TokenStream) -> TokenStream {
    let func: ItemFn = match parse2(input) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error(),
    };

    let name = func.sig.ident.clone();
    let vis = &func.vis;
    let body = &func.block;
    let props_ident = format_ident!("{}Props", name);

    let pattern = parse_layout_pattern(args);

    let trampoline = format_ident!("__resuma_layout_trampoline_{}", name);
    let registry = format_ident!("__resuma_layout_register_{}", name);
    let pattern_lit = pattern;

    quote! {
        #[allow(non_camel_case_types)]
        #vis struct #name;

        #[derive(Default, Clone)]
        #[allow(non_snake_case)]
        #vis struct #props_ident {
            #[doc(hidden)]
            pub __resuma_slotted: ::std::vec::Vec<::resuma::__private::SlottedChild>,
        }

        impl #props_ident {
            #[doc(hidden)]
            pub fn __resuma_slotted(mut self, c: ::std::vec::Vec<::resuma::__private::SlottedChild>) -> Self {
                self.__resuma_slotted = c;
                self
            }
        }

        impl ::resuma::__private::Component for #name {
            type Props = #props_ident;

            fn name() -> &'static str { stringify!(#name) }

            fn render(props: Self::Props) -> ::resuma::__private::View {
                let _slot_guard = ::resuma::__private::push_slots(props.__resuma_slotted);
                #body
            }
        }

        #[doc(hidden)]
        #[allow(non_snake_case)]
        fn #trampoline(req: ::resuma::FlowRequest, inner: ::resuma::__private::View) -> ::resuma::__private::View {
            let _req = req;
            ::resuma::__private::render_component::<#name>(
                <#name as ::resuma::__private::Component>::Props::default()
                    .__resuma_slotted(vec![::resuma::__private::SlottedChild {
                        slot: None,
                        child: ::resuma::__private::Child::View(inner),
                    }])
            )
        }

        #[doc(hidden)]
        #[allow(non_snake_case)]
        #[::resuma::__private::ctor::ctor]
        fn #registry() {
            ::resuma::register_layout(
                #pattern_lit,
                ::std::sync::Arc::new(#trampoline),
            );
        }
    }
}

fn parse_layout_pattern(args: TokenStream) -> TokenStream {
    if args.is_empty() {
        return quote! { "/" };
    }
    if let Ok(lit) = syn::parse2::<LitStr>(args.clone()) {
        let s = lit.value();
        return quote! { #s };
    }
    args
}
