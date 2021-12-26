use crate::{Elm, ElmJson};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
#[serde(tag = "t")]
enum Enum {
    Unit,
    Named { field: i32 },
}

#[test]
fn unit() {
    super::test(Enum::Unit);
}

#[test]
fn named() {
    super::test(Enum::Named { field: 123 });
}
