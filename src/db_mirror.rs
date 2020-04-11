use common::{struct_fields, filter_attributes};
use quote;
use syn::{Ident, Field};

pub fn impl_db_mirror(ast: &syn::DeriveInput) -> quote::Tokens {
    let mut queries = generate_insert_queries(ast);

    queries.append(generate_select_queries(ast));

    queries
}

fn generate_select_queries(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let fields = struct_fields(ast);
    let partition_key_fields = filter_attributes(&fields, "partition_key");

    let mut select_all = quote! {
        impl #name {
            pub fn select_all() -> &'static str {
                concat!("select * from ", stringify!(#name))
            }

            pub fn select_all_count() -> &'static str {
                concat!("select count(*) from ", stringify!(#name))
            }
        }
    };


    // How to implement this macro?
    // For each and every one
    let select_all_primary_key = quote! {
        impl #name {

         }
    };


    select_all.append(select_all_primary_key);

    select_all
}

fn generate_insert_queries(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let idents = struct_fields(ast)
        .iter()
        .map(|f| f.ident.clone().unwrap())
        .collect::<Vec<_>>();
    // TODO when https://github.com/AlexPikalov/cdrs-helpers-derive/issues/8 is merged,
    // this variable can be replaced by variable 'idents'
    let idents_copy = idents.clone();
    let fields_to_string = idents
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>();
    let names = fields_to_string
        .join(", ");
    let question_marks = fields_to_string
        .iter()
        .map(|_| "?".to_string()).collect::<Vec<String>>()
        .join(", ");

    quote! {
        impl #name {
            pub fn insert_query() -> &'static str {
                concat!("insert into ", stringify!(#name), "(",
                  #names,
                 ") values (",
                 #question_marks,
                 ")")
            }

            pub fn into_query_values(self) -> cdrs::query::QueryValues {
                use std::collections::HashMap;
                let mut values: HashMap<String, cdrs::types::value::Value> = HashMap::new();

                #(
                    values.insert(stringify!(#idents).to_string(), self.#idents_copy.into());
                )*

                cdrs::query::QueryValues::NamedValues(values)
            }
        }
    }
}