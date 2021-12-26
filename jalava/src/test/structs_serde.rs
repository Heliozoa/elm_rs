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
    super::test(Named {
        field: 123,
        renamed: 0,
    });
}
