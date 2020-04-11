use common::struct_fields;
use quote;

pub fn impl_db_mirror(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let idents = struct_fields(ast)
        .iter()
        .map(|f| f.ident.clone().unwrap())
        .collect::<Vec<_>>();
    // TODO when https://github.com/AlexPikalov/cdrs-helpers-derive/issues/8 is merged,
    // this variable can be replaced by variable 'idents'
    let idents_copy = idents.clone();

    let fields = idents
        .iter()
        .map(|i| i.to_string())
        .collect::<Vec<String>>();
    let names = fields
        .join(", ");
    let question_marks = fields
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
                    values.insert(stringify!(#idents), self.#idents_copy);
                )*

                cdrs::query::QueryValues::NamedValues(values)
            }
        }
    }
}