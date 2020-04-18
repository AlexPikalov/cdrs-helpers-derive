use common::get_struct_fields;
use quote;
use syn;

pub fn impl_try_from_udt(ast: &syn::DeriveInput) -> quote::Tokens {
  let name = &ast.ident;
  let fields = get_struct_fields(ast);
  quote! {
      impl TryFromUDT for #name {
        fn try_from_udt(cdrs: cdrs::types::udt::UDT) -> cdrs::Result<Self> {
          Ok(#name {
            #(#fields),*
          })
        }
      }
  }
}
