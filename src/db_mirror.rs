use quote;
use syn::{Ident, Field};

pub fn impl_db_mirror(ast: &syn::DeriveInput) -> quote::Tokens {
    let mut queries = ::db_mirror::select_queries::generate_select_queries(ast);

    queries.append(::db_mirror::insert_queries::generate_insert_queries(ast));

    queries
}

pub mod select_queries {
    use common::{struct_fields, filter_attributes};
    use syn::{Field, Ident, QSelf, Path, PathSegment, AngleBracketedParameterData, Ty};

    const COLUMN_SEPARATOR: &str = "_";

    struct SelectQueryWriter {
        name: Ident,

    }

    fn write(name: &Ident, v: &Vec<Field>, v_is_full_pk: bool) -> quote::Tokens {
        let names = v.iter().map(|f| f.ident.clone().unwrap()).collect::<Vec<_>>();
        let types = v.iter().map(|f| f.ty.clone()).collect::<Vec<_>>();

        let mut without_in = write_without_in(name, v, &names, &types, v_is_full_pk);
        let with_in = write_with_in(name, v, &names, &types);

        println!("with in: {}", with_in);

        without_in.append(with_in);

        without_in
    }

    fn write_without_in(name: &Ident, v: &Vec<Field>, names: &Vec<Ident>, types: &Vec<syn::Ty>, v_is_full_pk: bool) -> quote::Tokens {
        let where_clause = parameterized(&names);
        let fn_name = Ident::new(create_fn_name(&v, v_is_full_pk));

        write_impl(name, &names.clone(), names, types, fn_name, where_clause)
    }

    fn write_with_in(name: &Ident, v: &Vec<Field>, names: &Vec<Ident>, types: &Vec<syn::Ty>) -> quote::Tokens {
        let mut names_clone = names.clone();
        let mut types_clone = types.clone();
        let mut v = v.clone();

        // Remove the last element
        let last_name = names_clone.remove(names_clone.len() - 1);
        let last_type = types_clone.remove(types_clone.len() - 1);
        v.remove(v.len() - 1);

        let mut where_clause = parameterized(&names_clone);

        where_clause.push_str(&format!(" and {} in ?", last_name.as_ref()));

        let mut fn_name = create_fn_name(&v, false);

        let last_type_ident = match last_type {
            Ty::Path(_, p) => {
                p.segments[0].ident.clone()
            }
            _ => panic!()
        };

        types_clone.push(syn::Ty::Path(None, syn::Path::from(
            syn::PathSegment {
                ident: Ident::new("std::vec::Vec"),
                parameters: syn::PathParameters::AngleBracketed(AngleBracketedParameterData {
                    lifetimes: vec![],
                    types: vec![syn::Ty::Path(None,
                                              syn::Path {
                                                  global: false,
                                                  segments: vec![syn::PathSegment {
                                                      ident: last_type_ident,
                                                      parameters: syn::PathParameters::AngleBracketed(AngleBracketedParameterData {
                                                          lifetimes: vec![],
                                                          types: vec![],
                                                          bindings: vec![],
                                                      }),
                                                  }],
                                              },
                    )],
                    bindings: vec![],
                }),
            }
        )));

        fn_name.push_str(&format!("{}in{}{}", COLUMN_SEPARATOR, COLUMN_SEPARATOR, last_name.as_ref()));
        names_clone.push(Ident::new(format!("{}_in", last_name)));

        write_impl(name, &names_clone, names, &types_clone, Ident::new(fn_name), where_clause)
    }

    fn write_impl(
        name: &Ident,
        // The names that will be used for the parameters
        param_names: &Vec<Ident>,
        // The keys that will be used for QueryValues
        // The only difference between this parameter and 'param_names', is in the case
        // of creating a fn with an 'in' clause. The last param_name does have a suffixed '_in',
        // but this parameter does not have the suffix.
        qv_names: &Vec<Ident>,
        types: &Vec<syn::Ty>,
        fn_name: Ident,
        where_clause: String
    ) -> quote::Tokens {
        // TODO when https://github.com/AlexPikalov/cdrs-helpers-derive/issues/8 is merged,
        // this those variables can be replaced by variable 'param_names'
        let param_names_copy = param_names.clone();

        quote! {
            impl #name {
                pub fn #fn_name(#(#param_names: #types),*) -> (&'static str, cdrs::query::QueryValues) {
                    use std::collections::HashMap;
                    let mut values: HashMap<String, cdrs::types::value::Value> = HashMap::new();

                    #(
                      values.insert(stringify!(#qv_names).to_string(), #param_names_copy.into());
                    )*

                    (concat!("select * from ", stringify!(#name), " where ", #where_clause), cdrs::query::QueryValues::NamedValues(values))
                }
            }
        }
    }

    fn create_fn_name(v: &Vec<Field>, is_unique: bool) -> String {
        if is_unique {
            return "select_unique".to_string();
        }

        "select_by_".to_string() + &v
            .iter()
            .map(|p| p.ident.clone().unwrap().to_string())
            .collect::<Vec<_>>()
            .join(COLUMN_SEPARATOR)
    }

    fn parameterized(v: &Vec<Ident>) -> String {
        v
            .iter()
            .map(|f| f.clone().to_string() + " = ?")
            .collect::<Vec<_>>()
            .join(" and ")
    }

    pub fn generate_select_queries(ast: &syn::DeriveInput) -> quote::Tokens {
        let name = &ast.ident;

        let mut select_all = quote! {
            impl # name {
                pub fn select_all() -> & 'static str {
                    concat ! ("select * from ", stringify ! ( # name))
                }

                pub fn select_all_count() -> & 'static str {
                    concat ! ("select count(*) from ", stringify ! ( # name))
                }
            }
        };

        let fields = struct_fields(ast).clone();
        let partition_key_fields = filter_attributes(&fields, "partition_key");
        let cluster_key_fields = filter_attributes(&fields, "clustering_key");

        if partition_key_fields.is_empty() {
            assert!(cluster_key_fields.is_empty());

            return select_all;
        }

        select_all.append(write(name, &partition_key_fields, cluster_key_fields.is_empty()));

        let mut processed_clustering_key_fields = partition_key_fields.clone();
        let key_size = partition_key_fields.len() + cluster_key_fields.len();

        for clustering_key in cluster_key_fields.iter() {
            processed_clustering_key_fields.push(clustering_key.clone());

            select_all.append(write(name, &processed_clustering_key_fields, processed_clustering_key_fields.len() == key_size))
        }

        select_all
    }
}

mod insert_queries {
    use common::{struct_fields, filter_attributes};

    pub fn generate_insert_queries(ast: &syn::DeriveInput) -> quote::Tokens {
        let name = &ast.ident;
        let idents = struct_fields(ast)
            .iter()
            .map(|f| f.ident.clone().unwrap())
            .collect::<Vec<_>>();
// TODO when https://github.com/AlexPikalov/cdrs-helpers-derive/issues/8 is merged,
// this variable can be replaced by variable 'idents'
        let idents_copy = idents.clone();
        let fields_to_string = idents
            .iter()
            .map(|i| i.to_string())
            .collect::<Vec<String>>();
        let names = fields_to_string
            .join(", ");
        let question_marks = fields_to_string
            .iter()
            .map(|_| "?".to_string()).collect::<Vec<String>>()
            .join(", ");

        quote! {
            impl # name {
                pub fn insert_query() -> & 'static str {
                    concat ! ("insert into ", stringify ! ( # name), "(",
                    # names,
                    ") values (",
                    #question_marks,
                    ")")
                }

            pub fn into_query_values( self ) -> cdrs::query::QueryValues {
                use std::collections::HashMap;
                let mut values: HashMap < String, cdrs::types::value::Value > = HashMap::new();

                # (
                values.insert(stringify ! ( #idents).to_string(), self.#idents_copy.into());
                ) *

                cdrs::query::QueryValues::NamedValues(values)
                }
            }
        }
    }
}