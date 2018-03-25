use syn;
use quote;

pub fn get_struct_fields(ast: &syn::DeriveInput) -> Vec<quote::Tokens> {
  if let syn::Body::Struct(syn::VariantData::Struct(ref fields)) = ast.body {
    let fields = fields.iter().map(|field| {
      let name = field.ident.clone().unwrap();
      let value = convert_field_into_rust(field.clone());
      quote!{
        #name: #value
      }
    });

    fields.collect()
  } else {
    panic!("#[derive(IntoCDRSValue)] is only defined for structs, not for enums!");
  }
}

pub fn get_map_params_string(ty: syn::Ty) -> (syn::Ty, syn::Ty) {
  match ty {
    syn::Ty::Path(_, syn::Path { segments, .. }) => match segments.last() {
      Some(&syn::PathSegment {
        parameters: syn::PathParameters::AngleBracketed(ref angle_bracketed_data),
        ..
      }) => {
        let braket_types = angle_bracketed_data.types.clone();
        (
          braket_types
            .first()
            .expect("Cannot define Option type")
            .clone(),
          braket_types
            .last()
            .expect("Cannot define Option type")
            .clone(),
        )
      }
      _ => panic!("Cannot infer field type"),
    },
    _ => panic!("Cannot infer field type {:?}", ty),
  }
}

fn convert_field_into_rust(field: syn::Field) -> quote::Tokens {
  let mut string_name = quote!{};
  string_name.append("\"");
  string_name.append(field.ident.clone().unwrap());
  string_name.append("\".trim()");
  let arguments = get_arguments(string_name.clone());

  into_rust_with_args(field.ty, arguments)
}

fn get_arguments(name: quote::Tokens) -> quote::Tokens {
  quote! {
    &cdrs, #name
  }
}

fn into_rust_with_args(field_type: syn::Ty, arguments: quote::Tokens) -> quote::Tokens {
  let field_type_ident = get_cdrs_type_ident(field_type.clone());
  match field_type_ident.as_ref() {
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
      quote! {
        #field_type_ident::from_cdrs_r(#arguments)?
      }
    }
    "List" => {
      let list_as_rust = as_rust(field_type, quote! {list});

      quote! {
        match List::from_cdrs_r(#arguments) {
          Ok(ref list) => {
            #list_as_rust
          },
          _ => return Err("List should not be empty".into())
        }
      }
    }
    "Map" => {
      let map_as_rust = as_rust(field_type, quote! {map});
      quote! {
        match Map::from_cdrs_r(#arguments) {
          Ok(map) => {
            #map_as_rust
          },
          _ => return Err("Map should not be empty".into())
        }
      }
    }
    "Option" => {
      let opt_type = get_ident_params_string(field_type.clone());
      let opt_type_rustified = get_cdrs_type_ident(opt_type.clone());
      let opt_value_as_rust = as_rust(opt_type.clone(), quote! {opt_value});
      let q = quote! {
        {
          match #opt_type_rustified::from_cdrs_by_name(#arguments)? {
          Some(opt_value) => {
            let decoded = #opt_value_as_rust;
            Some(decoded)
          },
          _ => None
        }
        }
      };

      println!(">> {:?}", q);

      q
    }
    _ => {
      quote! {
        #field_type::try_from_udt(UDT::from_cdrs_r(#arguments)?)?
      }
    }
  }
}

fn get_cdrs_type_ident(ty: syn::Ty) -> syn::Ident {
  let type_string = get_ident_string(ty.clone());
  match type_string.as_str() {
    "Blob" => "Blob".into(),
    "String" => "String".into(),
    "bool" => "bool".into(),
    "i64" => "i64".into(),
    "i32" => "i32".into(),
    "i16" => "i16".into(),
    "i8" => "i8".into(),
    "f64" => "f64".into(),
    "f32" => "f32".into(),
    "IpAddr" => "IpAddr".into(),
    "Uuid" => "Uuid".into(),
    "Timespec" => "Timespec".into(),
    "Vec" => "List".into(),
    "HashMap" => "Map".into(),
    "Option" => "Option".into(),
    _ => "UDT".into(),
  }
}

fn get_ident(ty: syn::Ty) -> syn::Ident {
  match ty {
    syn::Ty::Path(_, syn::Path { segments, .. }) => match segments.last() {
      Some(&syn::PathSegment { ref ident, .. }) => ident.clone(),
      _ => panic!("Cannot infer field type"),
    },
    _ => panic!("Cannot infer field type {:?}", ty),
  }
}

// returns single value decoded and optionaly iterative mapping that uses decoded value
fn as_rust(ty: syn::Ty, val: quote::Tokens) -> quote::Tokens {
  let cdrs_type = get_cdrs_type_ident(ty.clone());
  match cdrs_type.as_ref() {
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
    "Timespec" => val,
    "List" => {
      let vec_type = get_ident_params_string(ty.clone());
      let inter_rust_type = get_cdrs_type_ident(vec_type.clone());
      let decoded_item = as_rust(vec_type.clone(), quote! {item});
      quote! {
        {
          let inner: Vec<#inter_rust_type> = #val.as_rust_type()?.unwrap();
          let mut decoded: Vec<#vec_type> = Vec::with_capacity(inner.len());
          for item in inner {
            decoded.push(#decoded_item);
          }
          decoded
        }
      }
    },
    "Map" => {
      let (map_key_type, map_value_type) = get_map_params_string(ty.clone());
      let inter_rust_type = get_cdrs_type_ident(map_value_type.clone());
      let decoded_item = as_rust(map_value_type.clone(), quote! {val});
      quote! {
        {
          let inner: HashMap<#map_key_type, #inter_rust_type> = #val.as_rust_type()?.unwrap();
          let mut decoded: HashMap<#map_key_type, #map_value_type> = HashMap::with_capacity(inner.len());
          for (key, val) in inner {
            decoded.insert(key, #decoded_item);
          }
          decoded
        }
      }
    },
    "Option" => {
      let opt_type = get_ident_params_string(ty.clone());
      as_rust(opt_type.clone(), val)
    },
    _ => {
      quote! {
        #ty::try_from_udt(#val)?
      }
    }
  }
}

pub fn get_ident_string(ty: syn::Ty) -> String {
  get_ident(ty).as_ref().into()
}

pub fn get_ident_params_string(ty: syn::Ty) -> syn::Ty {
  match ty {
    syn::Ty::Path(_, syn::Path { segments, .. }) => match segments.last() {
      Some(&syn::PathSegment {
        parameters: syn::PathParameters::AngleBracketed(ref angle_bracketed_data),
        ..
      }) => angle_bracketed_data
        .types
        .last()
        .expect("Cannot define Option type")
        .clone(),
      _ => panic!("Cannot infer field type"),
    },
    _ => panic!("Cannot infer field type {:?}", ty),
  }
}
