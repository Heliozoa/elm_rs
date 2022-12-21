use crate::{Elm, ElmDecode, ElmEncode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Elm, ElmEncode, ElmDecode, PartialEq)]
#[serde(tag = "type")]
enum Msg {
    Unit1,
    Unit2,
    Fields1 { field: i32 },
    Fields2 { field: i32 },
}

#[test]
fn regression_2() {
    super::test_json(Msg::Unit1);
}
