use crate::{Elm, ElmDecode, ElmEncode};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmEncode, ElmDecode)]
#[serde(tag = "t", content = "c")]
enum Enum {
    Unit1,
    Unit2,
    Newtype1(i32),
    Newtype2(i32),
    Tuple1(i32, i32),
    Tuple2(i32, i32),
    Named1 { field: i32 },
    Named2 { field: i32 },
}

#[test]
fn unit() {
    super::test_json(Enum::Unit1);
}

#[test]
fn newtype() {
    super::test_json(Enum::Newtype1(123));
}

#[test]
fn tuple() {
    super::test_json(Enum::Tuple1(123, 234));
}

#[test]
fn named() {
    super::test_json(Enum::Named1 { field: 123 });
}
