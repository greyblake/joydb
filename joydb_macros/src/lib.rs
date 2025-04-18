//! Supporting macros for `joydb` crate.
//!
//! Don’t use this crate directly, use `joydb` instead.
//!
//! For more information please refer to [joydb](https://docs.rs/joydb) documentation.

use proc_macro::TokenStream;

mod model;

#[proc_macro_derive(Model)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    crate::model::derive_model(input)
}
