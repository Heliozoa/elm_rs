use crate::{Elm, ElmJson};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
struct Unit;

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
struct Newtype(i32);

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
struct Tuple(i32, i32);

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
struct Named {
    field: i32,
}

#[test]
fn unit() {
    super::test(Unit);
}

#[test]
fn newtype() {
    super::test(Newtype(123));
}

#[test]
fn tuple() {
    super::test(Tuple(123, 234));
}

#[test]
fn named() {
    super::test(Named { field: 123 });
}
