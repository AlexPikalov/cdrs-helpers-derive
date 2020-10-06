use std::collections::HashMap;
use cdrs::query_values;

#[derive(Clone, Debug, cdrs_helpers_derive::IntoCDRSValue, cdrs_helpers_derive::TryFromRow)]
struct Udt {
    pub number: i32,
    pub number_16: i16,
    pub vec: Vec<Vec<i32>>,
    pub map: HashMap<i64, N>,
    pub opt: Option<HashMap<i64, N>>,
    pub uuid: uuid::Uuid
}

#[derive(Clone, Debug, cdrs_helpers_derive::IntoCDRSValue, cdrs_helpers_derive::TryFromUDT)]
struct N {
    pub n: i16,
    pub x: X,
}

#[derive(Clone, Debug, cdrs_helpers_derive::IntoCDRSValue, cdrs_helpers_derive::TryFromUDT)]
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
        uuid: Default::default()
    };
    let val: cdrs::types::value::Value = udt.clone().into();
    let values = query_values!(udt.clone());
    println!("as value {:?}", val);
    println!("among values {:?}", values);
}