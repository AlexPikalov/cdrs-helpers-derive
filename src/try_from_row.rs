use syn;
use quote;
use common::get_struct_fields;

pub fn impl_try_from_row(ast: &syn::DeriveInput) -> quote::Tokens {
  let name = &ast.ident;
  let fields = get_struct_fields(ast);

  quote! {
      impl TryFromRow for #name {
        fn try_from_row(cdrs: Row) -> cdrs::error::Result<Self> {
          Ok(#name {
            #(#fields),*
          })
        }
      }
  }
}
