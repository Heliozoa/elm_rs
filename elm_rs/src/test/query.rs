use crate::{Elm, ElmDecode, ElmEncode, ElmQuery, ElmQueryField};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmDecode, ElmEncode, ElmQuery)]
struct Named {
    first: i32,
    second: String,
    e: Enum,
}

#[test]
fn query_struct() {
    super::test_query::<_, Enum>(
        Named {
            first: 123,
            second: "234".to_string(),
            e: Enum::First,
        },
        "?first=123&second=234&e=First",
    );
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmDecode, ElmEncode, ElmQuery)]
struct ContainsEnum {
    e: Enum,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmDecode, ElmEncode, ElmQueryField)]
enum Enum {
    First,
    Second,
}

#[test]
fn query_enum() {
    super::test_query::<_, Enum>(ContainsEnum { e: Enum::First }, "?e=First");
}
