use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemStruct, parse_quote};

#[proc_macro_attribute]
pub fn tosql(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut input = parse_macro_input!(input as ItemStruct);
    input
        .attrs
        .push(parse_quote!(#[derive(tosql::ToSql, Debug)]));
    quote!(#input).into()
}
