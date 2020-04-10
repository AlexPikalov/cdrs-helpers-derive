use common::{get_ident_string, struct_fields};
use quote;
use syn;

pub fn impl_db_mirror(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let fields = struct_fields(ast);

    quote! {
        impl #name {
            pub fn insert_query() -> &'static str {
                concat!("insert into ", stringify!(#name), "(",
                  #(stringify!(#fields))*
                 , ") values (",
                 // Add the amount of '?' for the amount of fields
                 ")")
            }
        }
    }
}

