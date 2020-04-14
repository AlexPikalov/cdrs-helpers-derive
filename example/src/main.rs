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
// TODO: bij dates => enzo toevoegen, misschien ook bij andere parameters

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

    fn generate_some_struct() -> SomeStruct {
        SomeStruct {
            id: 1,
            another_id: 2,
            cluster_key: 3,
            another_cluster_key: 4,
            name: "name".to_string()
        }
    }

    #[test]
    fn test_select_queries() {
        // General select queries
        assert_eq!("select * from SomeStruct", SomeStruct::select_all());
        assert_eq!("select count(*) from SomeStruct", SomeStruct::select_all_count());

        // Queries with parameters
        let some_struct = generate_some_struct();
        // The line below should NOT be compiled, since only rows in a where clause can be queried by there full partition key
        // TODO: Maybe the trybuild crate can verify the non-compiling code?
        //assert_eq!("select * from SomeStruct where id = ?", SomeStruct::select_by_id());
        let (query, qv) = SomeStruct::select_by_id_another_id(some_struct.id, some_struct.another_id);

        assert_eq!("select * from SomeStruct where id = ? and another_id = ?", query);
        assert_eq!(query_values!("id" => some_struct.id, "another_id" => some_struct.another_id), qv);
        assert_eq!("select * from SomeStruct where id = ? and another_id = ? and cluster_key = ?", SomeStruct::select_by_id_another_id_cluster_key(1, 1, 1).0);
        assert_eq!("select * from SomeStruct where id = ? and another_id = ? and cluster_key = ? and another_cluster_key = ?", SomeStruct::select_unique(1, 1, 1, 1).0);

        // Queries with IN
        let vec = vec![1, 2];
        let v: Value = vec.clone().into();
        let (query, qv) = SomeStruct::select_by_id_another_id_in_cluster_key(some_struct.id, some_struct.another_id, vec.clone());

        assert_eq!("select * from SomeStruct where id = ? and another_id = ? and cluster_key in ?", query);
        assert_eq!(query_values!("id" => some_struct.id, "another_id" => some_struct.another_id, "cluster_key" => v), qv);
        assert_eq!("select * from SomeStruct where id = ? and another_id = ? and cluster_key = ? and another_cluster_key in ?",
                   SomeStruct::select_by_id_another_id_cluster_key_in_another_cluster_key(1, 1, 1, vec).0);
    }

    #[test]
    fn test_select_range_queries() {
        let some_struct = generate_some_struct();

        let (query, qv) = SomeStruct::select_by_id_another_id_cluster_key_larger_than_another_cluster_key(some_struct.id, some_struct.another_id, some_struct.cluster_key, some_struct.another_cluster_key);

        assert_eq!("select * from SomeStruct where id = ? and another_id = ? and cluster_key = ? and another_cluster_key > ?", query);
        assert_eq!(query_values!("id" => some_struct.id, "another_id" => some_struct.another_id, "cluster_key" => some_struct.cluster_key, "another_cluster_key" => some_struct.another_cluster_key), qv);
    }

}