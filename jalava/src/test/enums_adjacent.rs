use crate::{Elm, ElmJson};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
#[serde(tag = "t", content = "c")]
enum Enum {
    Unit,
    Newtype(i32),
    Tuple(i32, i32),
    Named { field: i32 },
}

#[test]
fn unit() {
    super::test_json(Enum::Unit);
}

#[test]
fn newtype() {
    super::test_json(Enum::Newtype(123));
}

#[test]
fn tuple() {
    super::test_json(Enum::Tuple(123, 234));
}

#[test]
fn named() {
    super::test_json(Enum::Named { field: 123 });
}
