//! `js!{}` macro — escape hatch that takes any token stream and produces a
//! `HandlerSource::Inline(String)`. The string is later embedded into the
//! page (or a chunk) verbatim by the SSR layer.

use proc_macro2::TokenStream;
use quote::quote;

pub fn expand(input: TokenStream) -> TokenStream {
    // We round-trip the tokens through `to_string` to preserve the user's
    // literal JS source. Note: this is a best-effort — escapes inside string
    // literals are preserved by `proc_macro2` because we never re-tokenize.
    let source = input.to_string();
    quote! {
        ::resuma::__private::HandlerSource::Inline(#source.to_string())
    }
}
