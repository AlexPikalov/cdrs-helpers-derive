use common::get_struct_fields;
use quote;
use syn;

pub fn impl_try_from_row(ast: &syn::DeriveInput) -> quote::Tokens {
  let name = &ast.ident;
  let fields = get_struct_fields(ast);

  quote! {
      impl TryFromRow for #name {
        fn try_from_row(cdrs: cdrs::types::rows::Row) -> cdrs::Result<Self> {
          Ok(#name {
            #(#fields),*
          })
        }
      }
  }
}
