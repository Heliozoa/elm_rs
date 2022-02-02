use crate::{Elm, ElmJson};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
#[serde(rename = "renamed")]
struct Unit;

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
#[serde(rename = "renamed")]
struct Newtype(i32);

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
#[serde(rename = "renamed")]
struct Tuple(i32, i32);

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
#[serde(rename = "renamed")]
#[serde(rename_all = "UPPERCASE")]
struct Named {
    field: i32,
    #[serde(rename = "another-field")]
    renamed: i32,
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
        field: 123,
        renamed: 0,
    });
}
