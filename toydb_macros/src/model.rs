use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::{Data, DeriveInput, Field, Fields};

pub fn derive_model(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    expand_to_derive_model(input)
        .unwrap_or_else(|e| syn::Error::to_compile_error(&e))
        .into()
}

fn expand_to_derive_model(input: proc_macro::TokenStream) -> Result<TokenStream, syn::Error> {
    let derive_input: DeriveInput = syn::parse(input)?;
    let model = parse_model_struct(&derive_input)?;
    Ok(gen_derive_model(&model))
}

struct Model {
    type_name: Ident,
    id_field: Field,
}

// At the moment id field is hardcoded to be `id`, but could be changed in the future to be
// parameterized if necessary.
const ID_NAME: &str = "id";

fn parse_model_struct(input: &DeriveInput) -> Result<Model, syn::Error> {
    const BAD_TYPE_ERROR_MSG: &str = "Model must be a struct with named fields";
    let Data::Struct(data_struct) = &input.data else {
        return Err(syn::Error::new(input.ident.span(), BAD_TYPE_ERROR_MSG));
    };

    let Fields::Named(fields) = &data_struct.fields else {
        return Err(syn::Error::new(input.ident.span(), BAD_TYPE_ERROR_MSG));
    };

    let id_field = fields
        .named
        .iter()
        .find(|field| {
            field
                .ident
                .as_ref()
                .map_or(false, |ident| ident.to_string() == ID_NAME)
        })
        .ok_or_else(|| syn::Error::new(input.ident.span(), "Model must have an `id` field"))?;

    Ok(Model {
        type_name: input.ident.clone(),
        id_field: id_field.clone(),
    })
}

fn gen_derive_model(model: &Model) -> TokenStream {
    let Model {
        type_name,
        id_field,
    } = model;

    let id_field_type = &id_field.ty;
    let id_field_name = id_field
        .ident
        .as_ref()
        .expect("id field name is guaranteed to be present");

    quote! {
        impl ::toydb::Model for #type_name {    // impl ::toydb::Model for User {
            type Id = #id_field_type;           //     type Id = i32
                                                //
            fn id(&self) -> &Self::Id {         //     fn id(&self) -> Self::Id {
                &self.#id_field_name            //         &self.id
            }                                   //     }
        }                                       // }
    }
}
