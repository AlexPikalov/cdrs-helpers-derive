// use syn;
// use quote;


// /// It receives Rust structure and returns a vector of tokens - one per each field in the structure.
// pub fn get_struct_fields(ast: &syn::DeriveInput) -> Vec<quote::Tokens> {
//   if let syn::Body::Struct(syn::VariantData::Struct(ref fields)) = ast.body {
//     let fields = fields.iter().map(|field| {
//       let name = field.ident.clone().unwrap();
//       let value = convert_field_into_rust(field.clone());
//       quote!{
//         #name: #value
//       }
//     });

//     fields.collect()
//   } else {
//     panic!("#[derive(IntoCDRSValue)] is only defined for structs, not for enums!");
//   }
// }

// fn convert_field_into_rust(field: syn::Field) -> quote::Tokens {
//   let mut string_name = quote!{};
//   string_name.append("\"");
//   string_name.append(field.ident.clone().unwrap());
//   string_name.append("\".trim()");
//   let arguments = get_arguments(string_name.clone());

//   into_rust_with_args(field.ty, arguments)
// }
