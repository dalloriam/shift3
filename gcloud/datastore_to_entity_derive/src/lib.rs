extern crate proc_macro;

use proc_macro::TokenStream;

use quote::quote;

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

        impl ToEntity for #type_name {

            #[inline]
            fn get_kind() -> &'static str {
                const _ENTITY_KIND: &str = stringify!(#type_name);
                _ENTITY_KIND
            }

            fn into_entity(self) -> gcloud::datastore::DSEntity {
                let mut entity_data = std::collections::HashMap::new();
                #(
                    entity_data.insert(String::from(stringify!(#struct_fields)), gcloud::datastore::DatastoreValue::from(self.#struct_fields));
                )*
                gcloud::datastore::DSEntity{entity_data, entity_kind: String::from(Self::get_kind()), entity_name: None}
            }

            fn from_entity(mut ds: gcloud::datastore::DSEntity) -> Self {
                // TODO: Remove unwraps.
                Self{
                    #(
                        #struct_fields: std::convert::TryInto::try_into(ds.entity_data.remove(stringify!(#struct_fields)).unwrap()).unwrap(),
                    )*
                }
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
