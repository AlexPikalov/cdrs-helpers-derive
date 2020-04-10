use common::struct_fields;
use quote;
use syn;
use syn::Ident;

pub fn impl_db_mirror(ast: &syn::DeriveInput) -> quote::Tokens {
    let name = &ast.ident;
    let fields = struct_fields(ast);
    let fields: Vec<Ident> = fields.iter().map(|f| f.ident.clone().unwrap()).collect();

    quote! {
        impl #name {
            pub fn insert_query() -> &'static str {
                concat!("insert into ", stringify!(#name), "(",
                  #(stringify!(#fields, ),)*
                 ") values (",
                 #("?"),*
                 ")")
            }
        }
    }
}

