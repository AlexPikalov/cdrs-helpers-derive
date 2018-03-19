use syn;
use quote;

pub fn impl_into_cdrs_value(ast: &syn::DeriveInput) -> quote::Tokens {
  let name = &ast.ident;
  if let syn::Body::Struct(syn::VariantData::Struct(ref fields)) = ast.body {
    let conver_into_bytes = fields
      .iter()
      .map(|field| field.ident.clone().unwrap())
      .map(|field| {
        quote! {
            let field_value = self.#field.into_cdrs_value();
            bytes.extend_from_slice(field_value.into_cbytes().as_slice());
        }
      });

    quote! {
        impl IntoCDRSValue for #name {
            fn into_cdrs_value(self) -> Value {
                let mut bytes: Vec<u8> = vec![];
                #(#conver_into_bytes)*
                Bytes::new(bytes).into()
            }
        }
    }
  } else {
    panic!("#[derive(IntoCDRSValue)] is only defined for structs, not for enums!");
  }
}
