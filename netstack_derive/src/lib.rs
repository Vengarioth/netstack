extern crate proc_macro;

use proc_macro::TokenStream;
use syn::parse_macro_input;

mod serialize;
mod deserialize;

#[proc_macro_derive(Serialize, attributes(netstack))]
pub fn derive_serialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    serialize::derive_serialize_impl(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[proc_macro_derive(Deserialize, attributes(netstack))]
pub fn derive_deserialize(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as syn::DeriveInput);
    deserialize::derive_deserialize_impl(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}
