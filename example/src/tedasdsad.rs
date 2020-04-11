use cdrs::types::prelude::*;
use std::collections::HashMap;

// #[derive(Debug, IntoCDRSValue, TryFromRow)]
#[derive(Clone, Debug, TryFromRow)]
struct Udt {
    pub opt: Option<HashMap<i64, N>>
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