// lib.rs
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(DataComponent)]
pub fn derive_data_component(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Get the name of the struct
    let name = input.ident;

    // Generate the implementation of the DataComponent trait
    let expanded = quote! {
        impl DataComponent for #name {
            fn to_json(&self) -> String {
                serde_json::to_string(self).unwrap()
            }

            fn from_json(json: &str) -> Self {
                serde_json::from_str(json).unwrap()
            }
        }
    };

    // Convert the expanded code into a token stream
    TokenStream::from(expanded)
}
