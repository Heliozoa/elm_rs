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
    first: i32,
    second: String,
}

#[test]
fn unit() {
    super::test_json(Unit);
}

#[test]
fn newtype() {
    super::test_json(Newtype(123));
}

#[test]
fn tuple() {
    super::test_json(Tuple(123, 234));
}

#[test]
fn named() {
    super::test_json(Named {
        first: 123,
        second: "234".to_string(),
    });
}
