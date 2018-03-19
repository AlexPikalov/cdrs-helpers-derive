extern crate cdrs;
#[macro_use]
extern crate cdrs_helpers_derive;

use std::collections::HashMap;
use cdrs::types::AsRust;
use cdrs::types::value::{Bytes, Value};
use cdrs::frame::{IntoBytes, IntoCDRSValue, TryFromRow, TryFromUDT};
use cdrs::types::rows::Row;
use cdrs::types::udt::UDT;
use cdrs::types::list::List;
use cdrs::types::map::Map;
use cdrs::types::from_cdrs::FromCDRSByName;

#[derive(Debug, IntoCDRSValue, TryFromRow)]
struct Udt {
    pub number: i32,
    pub number_16: i16,
    // pub tuple: (i32, i32),
    pub vec: Vec<i8>,
    pub map: HashMap<String, i8>, // pub paren: (i8),
    pub number_8: N,
}

#[derive(Debug, IntoCDRSValue, TryFromRow, TryFromUDT)]
struct N {
    pub n: i16,
}

fn main() {
    let udt = Udt {
        number: 12,
        number_16: 256,
        vec: vec![1, 2, 3],
        map: HashMap::new(),
        number_8: N { n: 0 },
    };
    let val: Value = udt.into_cdrs_value();
    println!("values {:?}", val);
}
