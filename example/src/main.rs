#[macro_use]
extern crate cdrs;
#[macro_use]
extern crate cdrs_helpers_derive;
extern crate time;

use time::Timespec;
use std::collections::HashMap;
use cdrs::types::AsRustType;
use cdrs::types::value::{Bytes, Value};
use cdrs::frame::{IntoBytes, TryFromRow, TryFromUDT};
use cdrs::types::rows::Row;
use cdrs::types::udt::UDT;
use cdrs::types::list::List;
use cdrs::types::map::Map;
use cdrs::types::from_cdrs::FromCDRSByName;

// #[derive(Debug, IntoCDRSValue, TryFromRow)]
#[derive(Clone, Debug, IntoCDRSValue, TryFromRow)]
struct Udt {
    pub number: i32,
    pub number_16: i16,
    // pub vec: Vec<Vec<N>>,
    pub vec: Vec<Vec<i32>>,
    pub map: HashMap<i64, N>,
    pub opt: Option<HashMap<i64, N>>,
    pub my_timestamp: Option<Timespec>,
}

// #[derive(Debug, IntoCDRSValue, TryFromRow, TryFromUDT)]
#[derive(Clone, Debug, IntoCDRSValue, TryFromUDT)]
struct N {
    pub n: i16,
    pub x: X,
}

#[derive(Clone, Debug, IntoCDRSValue, TryFromUDT)]
struct X {
    pub n: i32,
}

fn main() {
    let udt = Udt {
        number: 12,
        number_16: 256,
        vec: vec![vec![1, 2]],
        map: HashMap::new(),
        opt: Some(HashMap::new()),
        my_timestamp: None,
    };
    let val: cdrs::types::value::Value = udt.clone().into();
    let values = query_values!(udt.clone());
    println!("as value {:?}", val);
    println!("among values {:?}", values);
}

#[cfg(test)]
mod test_db_mirror {
    use cdrs::query::QueryValues;
    use cdrs::types::prelude::Value;

    #[derive(DBMirror)]
    #[allow(dead_code)]
    struct SomeStruct {
        #[partition_key]
        id: i32,
        #[partition_key]
        another_id: i32,
        #[clustering_key]
        cluster_key: i32,
        #[clustering_key]
        another_cluster_key: i32,
        // Just some column that is not part of the primary key
        name: String,
    }

    #[test]
    fn test_insert_query() {
        assert_eq!("insert into SomeStruct(id, another_id, cluster_key, another_cluster_key, name) values (?, ?, ?, ?, ?)", SomeStruct::insert_query())
    }

    #[test]
    fn test_into_query_values() {
        let id = 1;
        let cluster_key = 3;
        let name = "some name".to_string();

        let query_values: QueryValues = SomeStruct {
            id,
            another_id: id,
            cluster_key,
            another_cluster_key: cluster_key,
            name: name.clone(),
        }.into_query_values();

        if let QueryValues::NamedValues(nv) = query_values {
            assert_eq!(5, nv.len());

            let id_val: Value = id.into();
            assert_eq!(&id_val, nv.get("id").unwrap());

            let cluster_key: Value = cluster_key.into();
            assert_eq!(&cluster_key, nv.get("cluster_key").unwrap());

            let name_val: Value = name.into();
            assert_eq!(&name_val, nv.get("name").unwrap());
        } else {
            panic!("Expected named values");
        }
    }
}