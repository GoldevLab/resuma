//! `#[island]` — like `#[component]` but the compiler also emits a JS chunk
//! containing every event handler registered by the component, so the
//! browser can resume interactivity without reloading server-only code.
//!
//! Implementation note: the actual chunk emission happens at SSR time via
//! the `RenderContext`, which already collects every handler. The macro
//! only needs to mark the resulting `View` as an `Island` so the SSR layer
//! knows to wrap it in a `<resuma-island>` boundary.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemFn};

pub fn expand(_args: TokenStream, input: TokenStream) -> TokenStream {
    let func: ItemFn = match parse2(input) {
        Ok(f) => f,
        Err(e) => return e.to_compile_error(),
    };

    let name = func.sig.ident.clone();
    let name_str = name.to_string();
    let vis = func.vis.clone();
    let sig = func.sig.clone();
    let block = func.block.clone();

    quote! {
        #vis #sig {
            let __island_id = ::resuma::__private::current_context()
                .map(|c| c.next_signal_id().0)
                .unwrap_or(0);
            let __view: ::resuma::__private::View = (|| #block)();
            ::resuma::__private::wrap_in_island(#name_str, __island_id, __view)
        }
    }
}
