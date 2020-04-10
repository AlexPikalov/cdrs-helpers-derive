use common::struct_fields;
use quote;

pub fn impl_db_mirror(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let fields = struct_fields(ast)
        .iter()
        .map(|f| f.ident.clone().unwrap())
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
        }
    }
}