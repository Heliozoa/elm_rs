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

#[derive(Debug, Elm, ElmEncode, ElmDecode, Serialize, Deserialize, PartialEq)]

enum E {
    V { a: Option<i32> },
}

#[test]
fn regression_4() {
    super::test_json_with_deps(
        E::V { a: Some(1234) },
        &format!(
            "\
{}

{}

{}
",
            E::elm_definition().unwrap(),
            E::encoder_definition().unwrap(),
            E::decoder_definition().unwrap(),
        ),
    )
}
