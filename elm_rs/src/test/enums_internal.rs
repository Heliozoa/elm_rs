use crate::{Elm, ElmDecode, ElmEncode};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmEncode, ElmDecode)]
#[serde(tag = "t")]
enum Enum {
    Unit1,
    Unit2,
    Named1 { field: i32 },
    Named2 { field: i32 },
}

#[test]
fn unit() {
    super::test_json(Enum::Unit1);
}

#[test]
fn named() {
    super::test_json(Enum::Named1 { field: 123 });
}
