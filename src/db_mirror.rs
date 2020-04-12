use quote;
use syn::{Ident, Field};

pub fn impl_db_mirror(ast: &syn::DeriveInput) -> quote::Tokens {
    let mut queries = ::db_mirror::select_queries::generate_select_queries(ast);

    queries.append(::db_mirror::insert_queries::generate_insert_queries(ast));

    queries
}

pub mod select_queries {
    use common::{struct_fields, filter_attributes};
    use syn::{Field, Ident};

    fn write(name: &Ident, v: &Vec<Field>, v_is_full_pk: bool) -> quote::Tokens {
        let names = v.iter().map(|f| f.ident.clone().unwrap()).collect::<Vec<_>>();
        let types = v.iter().map(|f| f.ty.clone()).collect::<Vec<_>>();
        let parameterized = parameterized(&names);
        let fn_name = create_fn_name(&v, v_is_full_pk);

        // TODO when https://github.com/AlexPikalov/cdrs-helpers-derive/issues/8 is merged,
        // this those variables can be replaced by variable 'names'
        let names_copy = names.clone();
        let names_copy_2 = names.clone();

        quote! {
            impl #name {
                pub fn #fn_name(#(#names: #types),*) -> (&'static str, cdrs::query::QueryValues) {
                    use std::collections::HashMap;
                    let mut values: HashMap<String, cdrs::types::value::Value> = HashMap::new();

                    #(
                        values.insert(stringify!(#names_copy).to_string(), #names_copy_2.into());
                    )*

                    (concat!("select * from ", stringify!(#name), " where ", #parameterized), cdrs::query::QueryValues::NamedValues(values))
                }
            }
        }
    }

    fn create_fn_name(v: &Vec<Field>, is_unique: bool) -> Ident {
        if is_unique {
            return Ident::new("select_unique");
        }

        Ident::new("select_by_".to_string() + &v
            .iter()
            .map(|p| p.ident.clone().unwrap().to_string())
            .collect::<Vec<_>>()
            .join("_"))
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
        impl #name {
            pub fn select_all() -> &'static str {
                concat!("select * from ", stringify!(#name))
            }

            pub fn select_all_count() -> &'static str {
                concat!("select count(*) from ", stringify!(#name))
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
            impl #name {
                pub fn insert_query() -> &'static str {
                    concat!("insert into ", stringify!(#name), "(",
                      #names,
                     ") values (",
                     #question_marks,
                     ")")
                }

                pub fn into_query_values(self) -> cdrs::query::QueryValues {
                    use std::collections::HashMap;
                    let mut values: HashMap<String, cdrs::types::value::Value> = HashMap::new();

                    #(
                        values.insert(stringify!(#idents).to_string(), self.#idents_copy.into());
                    )*

                    cdrs::query::QueryValues::NamedValues(values)
                }
            }
        }
    }
}