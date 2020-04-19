use quote;
use syn::{Ident, Field};
use common::{struct_fields, filter_attributes};

pub fn impl_db_mirror(ast: &syn::DeriveInput) -> quote::Tokens {
    let mut queries = ::db_mirror::select_queries::generate_select_queries(ast);

    queries.append(::db_mirror::insert_queries::generate_insert_queries(ast));
    queries.append(::db_mirror::pk_object::generate_pk_object(ast));

    queries
}

fn read_with_attributes(ast: &syn::DeriveInput) -> (Vec<Field>, Vec<Field>, Vec<Field>) {
    let fields = struct_fields(ast).clone();
    let partition_key_fields = filter_attributes(&fields, "partition_key");
    let cluster_key_fields = filter_attributes(&fields, "clustering_key");

    if partition_key_fields.is_empty() {
        assert!(cluster_key_fields.is_empty());
    }

    (fields, partition_key_fields, cluster_key_fields)
}

pub mod pk_object {
    use db_mirror::read_with_attributes;
    use syn::Ident;
    use cs_ty_to_rs_ty::cdrs_ty_to_rust_ty;

    pub fn generate_pk_object(ast: &syn::DeriveInput) -> quote::Tokens {
        let name = &ast.ident;
        let (fields, mut partition_key_fields, mut cluster_key_fields) = read_with_attributes(ast);

        if partition_key_fields.is_empty() {
            return quote! { }
        }

        let struct_name = Ident::new(name.to_string() + "PrimaryKey");

        partition_key_fields.append(&mut cluster_key_fields);
        let idents = partition_key_fields
            .iter()
            .map(|p| p.ident.clone().unwrap())
            .collect::<Vec<_>>();
        // TODO: When https://github.com/AlexPikalov/cdrs-helpers-derive/issues/8 is merged,
        // this property 'types' and idents can be removed and be inlined
        let types = partition_key_fields
            .iter()
            .map(|p| p.ty.clone())
            .collect::<Vec<_>>();

        let mut properties = quote! { };
        let mut mapping = quote! { };

        for (index, ident) in idents.into_iter().enumerate() {
            let typee = types[index].clone();

            properties.append(quote! {
                pub #ident: #typee,
            });

            mapping.append(quote! {
               #ident: self.#ident.clone(),
            });
        };

        quote! {
            pub struct #struct_name {
                #properties
            }

            impl #name {
                pub fn primary_key(&self) -> #struct_name {
                    #struct_name {
                        #mapping
                    }
                }
            }
        }


    }
}

pub mod select_queries {
    use common::{struct_fields, filter_attributes};
    use syn::{Field, Ident, QSelf, Path, PathSegment, AngleBracketedParameterData, Ty};
    use quote::Tokens;
    use db_mirror::read_with_attributes;

    const COLUMN_SEPARATOR: &str = "_";

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

        let (fields, partition_key_fields, cluster_key_fields) = read_with_attributes(ast);

        if partition_key_fields.is_empty() {
            return select_all;
        }

        generate_all(
            &mut select_all,
            Writer::new(name, &partition_key_fields),
            cluster_key_fields.is_empty()
        );

        let mut processed_clustering_key_fields = partition_key_fields.clone();
        let key_size = partition_key_fields.len() + cluster_key_fields.len();

        for clustering_key in cluster_key_fields.iter() {
            processed_clustering_key_fields.push(clustering_key.clone());
            let writer = Writer::new(name, &processed_clustering_key_fields);

            generate_all(
                &mut select_all,
                writer.clone(),
                processed_clustering_key_fields.len() == key_size
            );

            for operator in Operator::all_operators() {
                select_all.append(generate(WriteRange::new(writer.clone(), operator)));
            }
        }

        select_all
    }

    fn generate_all(queries: &mut Tokens, writer: Writer, writing_full_pk: bool) {
        queries.append(generate(WriteWithoutIn {
            writer: writer.clone(),
            writing_full_pk,
        }));
        queries.append(generate(WriteWithIn::new(writer)))
    }

    #[derive(Debug, Clone)]
    struct Writer {
        name: Ident,
        fields: Vec<Field>,
        names: Vec<Ident>,
        types: Vec<Ty>,
    }

    impl Writer {
        fn new(name: &Ident, fields: &Vec<Field>) -> Self {
            Self {
                name: name.clone(),
                fields: fields.clone(),
                names: fields.iter().map(|f| f.ident.clone().unwrap()).collect::<Vec<_>>(),
                types: fields.iter().map(|f| f.ty.clone()).collect::<Vec<_>>(),
            }
        }
    }

    trait Write {
        fn name(&self) -> Ident;
        fn create_where_clause(&self) -> String;
        fn create_fn_name(&self) -> Ident;
        fn create_param_names(&self) -> Vec<Ident>;
        fn create_qv_names(&self) -> Vec<Ident>;
        fn create_types(&self) -> Vec<Ty>;
    }

    fn generate(write: impl Write) -> Tokens {
        write_impl(&write.name(), &write.create_param_names(), &write.create_qv_names(), &write.create_types(), write.create_fn_name(), write.create_where_clause())
    }

    struct WriteWithoutIn {
        writer: Writer,
        writing_full_pk: bool,
    }

    impl Write for WriteWithoutIn {
        fn name(&self) -> Ident {
            self.writer.name.clone()
        }

        fn create_where_clause(&self) -> String {
            parameterized(&self.writer.names)
        }

        fn create_fn_name(&self) -> Ident {
            if self.writing_full_pk {
                return Ident::new("select_unique");
            }

            Ident::new(create_fn_name(&self.writer.fields))
        }

        fn create_param_names(&self) -> Vec<Ident> {
            self.writer.names.clone()
        }

        fn create_qv_names(&self) -> Vec<Ident> {
            self.writer.names.clone()
        }

        fn create_types(&self) -> Vec<Ty> {
            self.writer.types.clone()
        }
    }

    enum Operator {
        // <=
        EqualWithSmaller,
        // <
        Smaller,
        // >=
        EqualWithLarger,
        // >
        Larger
    }

    impl Operator {
        fn all_operators() -> Vec<Operator> {
            vec![Operator::EqualWithSmaller, Operator::Smaller, Operator::EqualWithLarger, Operator::Larger]
        }
    }

    struct WriteRange {
        writer: Writer,
        last_field: Field,
        operator: Operator
    }

    impl WriteRange {
        fn new(writer: Writer, operator: Operator) -> Self {
            let write_with_in = WriteWithIn::new(writer);

            Self {
                writer: write_with_in.writer,
                last_field: write_with_in.last_field,
                operator
            }
        }
    }

    impl Write for WriteRange {
        fn name(&self) -> Ident {
            self.writer.name.clone()
        }

        fn create_where_clause(&self) -> String {
            let mut where_clause = parameterized(&self.writer.names);

            let op = match self.operator {
                Operator::EqualWithSmaller => "<=",
                Operator::Smaller => "<",
                Operator::EqualWithLarger => ">=",
                Operator::Larger => ">",
            };

            where_clause.push_str(&format!(" and {} {} ?", self.last_field.ident.clone().unwrap().as_ref(), op));

            where_clause
        }

        fn create_fn_name(&self) -> Ident {
            let mut fn_name = create_fn_name(&self.writer.fields);

            let separator = match self.operator {
                Operator::EqualWithSmaller => "equal_or_smaller_than",
                Operator::Smaller => "smaller_than",
                Operator::EqualWithLarger => "equal_or_larger_than",
                Operator::Larger => "larger_than",
            };

            fn_name.push_str(&format!("{}{}{}{}", COLUMN_SEPARATOR, separator, COLUMN_SEPARATOR, self.last_field.ident.clone().unwrap().as_ref()));

            Ident::new(fn_name)
        }

        fn create_param_names(&self) -> Vec<Ident> {
            let mut names = self.writer.names.clone();

            names.push(Ident::new(format!("range{}{}", COLUMN_SEPARATOR, self.last_field.ident.clone().unwrap().as_ref())));

            names
        }

        fn create_qv_names(&self) -> Vec<Ident> {
            // self.writer.names is missing the name of the last field, since it is removed
            let mut names = self.writer.names.clone();

            names.push(self.last_field.ident.clone().unwrap());

            names
        }

        fn create_types(&self) -> Vec<Ty> {
            let mut types = self.writer.types.clone();

            types.push(self.last_field.ty.clone());

            types
        }
    }

    struct WriteWithIn {
        writer: Writer,
        last_field: Field,
    }

    impl WriteWithIn {
        fn new(mut writer: Writer) -> Self {
            let last_field = writer.fields.remove(writer.fields.len() - 1);

            Self {
                writer: Writer::new(&writer.name, &writer.fields),
                last_field,
            }
        }
    }

    impl Write for WriteWithIn {
        fn name(&self) -> Ident {
            self.writer.name.clone()
        }

        fn create_where_clause(&self) -> String {
            let mut where_clause = parameterized(&self.writer.names);

            where_clause.push_str(&format!(" and {} in ?", self.last_field.ident.clone().unwrap().as_ref()));

            where_clause
        }

        fn create_fn_name(&self) -> Ident {
            let mut fn_name = create_fn_name(&self.writer.fields);

            fn_name.push_str(&format!("{}in{}{}", COLUMN_SEPARATOR, COLUMN_SEPARATOR, self.last_field.ident.clone().unwrap().as_ref()));

            Ident::new(fn_name)
        }

        fn create_param_names(&self) -> Vec<Ident> {
            let mut names = self.writer.names.clone();

            names.push(Ident::new(format!("in{}{}", COLUMN_SEPARATOR, self.last_field.ident.clone().unwrap().as_ref())));

            names
        }

        fn create_qv_names(&self) -> Vec<Ident> {
            // self.writer.names is missing the name of the last field, since it is removed
            let mut names = self.writer.names.clone();

            names.push(self.last_field.ident.clone().unwrap());

            names
        }

        fn create_types(&self) -> Vec<Ty> {
            let mut types = self.writer.types.clone();

            // Wrap the last type inside a Vec<>
            // TODO: maybe there is a shorter way to do this
            let last_type_ident = match self.last_field.ty.clone() {
                Ty::Path(_, p) => {
                    p.segments[0].ident.clone()
                }
                _ => panic!()
            };

            types.push(syn::Ty::Path(None, syn::Path::from(
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

            types
        }
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
        types: &Vec<Ty>,
        fn_name: Ident,
        where_clause: String,
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

    fn create_fn_name(v: &Vec<Field>) -> String {
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