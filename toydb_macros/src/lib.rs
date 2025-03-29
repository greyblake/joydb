use proc_macro::TokenStream;

mod model;

#[proc_macro_derive(Model)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    crate::model::derive_model(input)
}
