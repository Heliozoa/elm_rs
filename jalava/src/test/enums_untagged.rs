use crate::{Elm, ElmJson};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
#[serde(untagged)]
enum Enum {
    Unit,
    Newtype(i32),
    Tuple(i32, i32),
    Named { field: i32 },
}

#[test]
fn unit() {
    super::test(Enum::Unit);
}

#[test]
fn newtype() {
    super::test(Enum::Newtype(123));
}

#[test]
fn tuple() {
    super::test(Enum::Tuple(123, 234));
}

#[test]
fn named() {
    super::test(Enum::Named { field: 123 });
}
