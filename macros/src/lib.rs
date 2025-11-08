mod execute;
use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input
};

use crate::execute::Execute;

#[proc_macro]
pub fn execute(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as Execute);
    quote! { #parsed_input }.into()
}