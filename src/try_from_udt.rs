use syn;
use quote;
use common::get_struct_fields;

pub fn impl_try_from_udt(ast: &syn::DeriveInput) -> quote::Tokens {
  let name = &ast.ident;
  let fields = get_struct_fields(ast);
  quote! {
      impl TryFromUDT for #name {
        fn try_from_udt(cdrs: UDT) -> cdrs::error::Result<Self> {
          Ok(#name {
            #(#fields),*
          })
        }
      }
  }
}
