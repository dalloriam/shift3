extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;

use syn;
use syn::{Data, DataStruct, Fields, Ident};

fn impl_to_entity(ast: &syn::DeriveInput) -> TokenStream {
    let type_name = &ast.ident;

    let fields = match &ast.data {
        Data::Struct(DataStruct {
            fields: Fields::Named(ref fields),
            ..
        }) => &fields.named,
        _ => panic!("expected a struct with named fields."),
    };

    let struct_fields: &Vec<&Option<Ident>> =
        &fields.iter().map(|ref field| &field.ident).collect();

    let gen = quote! {
        use std::collections::HashMap;
        use gcloud::datastore::{DatastoreValue, DSEntity};

        const _ENTITY_ID: &str = stringify!(#type_name);

        impl ToEntity for #type_name {

            fn into_entity(self) -> DSEntity {
                let mut entity_data = HashMap::new();
                #(
                    entity_data.insert(String::from(stringify!(#struct_fields)), DatastoreValue::from(self.#struct_fields));
                )*
                DSEntity{entity_data, entity_id: _ENTITY_ID}
            }
        }
    };

    gen.into()
}

#[proc_macro_derive(ToEntity)]
pub fn to_entity_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse(input).unwrap();

    impl_to_entity(&ast)
}
