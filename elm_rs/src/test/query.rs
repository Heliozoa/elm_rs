use crate::{Elm, ElmDecode, ElmEncode, ElmQuery};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmEncode, ElmDecode, ElmQuery)]
struct Named {
    first: i32,
    second: String,
}

#[test]
fn query_struct() {
    super::test_query(
        Named {
            first: 123,
            second: "234".to_string(),
        },
        "?first=123&second=234",
    );
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmEncode, ElmDecode, ElmQuery)]
enum Enum {
    First { first: i32, second: String },
    Second { third: i32, fourth: String },
}

#[test]
fn query_enum() {
    super::test_query(
        Enum::Second {
            third: 123,
            fourth: "234".to_string(),
        },
        "?third=123&fourth=234",
    );
}
