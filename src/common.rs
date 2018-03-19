use syn;
use quote;

fn get_ident_string(ty: syn::Ty) -> String {
  match ty {
    syn::Ty::Path(_, syn::Path { segments, .. }) => match segments.last() {
      Some(&syn::PathSegment { ref ident, .. }) => ident.as_ref().into(),
      _ => panic!("Cannot infer field type"),
    },
    _ => panic!("Cannot infer field type {:?}", ty),
  }
}

pub fn get_struct_fields(ast: &syn::DeriveInput) -> Vec<quote::Tokens> {
  if let syn::Body::Struct(syn::VariantData::Struct(ref fields)) = ast.body {
    let fields = fields.iter().map(|field| {
      let name = field.ident.clone().unwrap();
      let mut string_name = quote!{};
      string_name.append("\"");
      string_name.append(field.ident.clone().unwrap());
      string_name.append("\".trim()");
      let field_type = field.ty.clone();
      let field_type_string = get_ident_string(field.ty.clone());

      match field_type_string.as_str() {
        "Blob" |
        "String" |
        "bool" |
        "i64" |
        "i32" |
        "i16" |
        "i8" |
        "f64" |
        "f32" |
        "IpAddr" |
        "Uuid" |
        "Timespec" => {
          return quote! {
            #name: #field_type::from_cdrs_r(&cdrs, #string_name)?
          };
        }
        "Vec" => {
          return quote! {
            #name: List::from_cdrs_r(&cdrs, #string_name).and_then(|list| list.as_r_rust())?
          };
        }
        "HashMap" => {
          return quote! {
            #name: Map::from_cdrs_r(&cdrs, #string_name).and_then(|map| map.as_r_rust())?
          };
        }
        _ => {
          return quote! {
            #name: #field_type::try_from_udt(UDT::from_cdrs_r(&cdrs, #string_name)?)?
          };
        }
      }
    });

    fields.collect()
  } else {
    panic!("#[derive(IntoCDRSValue)] is only defined for structs, not for enums!");
  }
}
