use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

#[proc_macro_derive(DataComponent)]
pub fn derive_data_component(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Clone the identifier (struct name) so we can use it multiple times
    let name = input.ident.clone();

    // Ensure that the input is a struct and has fields
    if let Data::Struct(_) = input.data {
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
    } else {
        // If not a struct, return an error
        let error =
            syn::Error::new_spanned(input, "DataComponent can only be derived for structs.");
        TokenStream::from(error.to_compile_error())
    }
}
