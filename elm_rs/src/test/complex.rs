use crate::{Elm, ElmDecode, ElmEncode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Elm, ElmEncode, ElmDecode, Serialize, Deserialize, PartialEq)]
enum Enum1<T> {
    Unit11,
    Unit12,
    Newtype11(T),
    Newtype12(T),
    Tuple11(T, T),
    Tuple12(T, T),
    Named11 { t: T },
    Named12 { t: T },
}

#[derive(Debug, Elm, ElmEncode, ElmDecode, Serialize, Deserialize, PartialEq)]
enum Enum2<T> {
    Unit21,
    Unit22,
    Newtype21(T),
    Newtype22(T),
    Tuple21(T, T),
    Tuple22(T, T),
    Named21 { t: T },
    Named22 { t: T },
}

#[derive(Debug, Elm, ElmEncode, ElmDecode, Serialize, Deserialize, PartialEq)]
struct Struct<T> {
    unit: Enum1<T>,
    newtype: Enum1<T>,
    tuple: Enum1<T>,
    named: Enum1<T>,
    named_unit: Enum2<Enum1<T>>,
    named_newtype: Enum2<Enum1<T>>,
    named_tuple: Enum2<Enum1<T>>,
    named_named: Enum2<Enum1<T>>,
}

#[test]
fn complex() {
    super::test_json_with_deps(
        Struct {
            unit: Enum1::Unit11,
            newtype: Enum1::Newtype11(vec![1, 2, 3, 4]),
            tuple: Enum1::Tuple11(vec![1, 2, 3, 4], vec![1, 2, 3, 4]),
            named: Enum1::Named11 {
                t: vec![1, 2, 3, 4],
            },
            named_unit: Enum2::Named21 { t: Enum1::Unit11 },
            named_newtype: Enum2::Named21 {
                t: Enum1::Newtype11(vec![1, 2, 3, 4]),
            },
            named_tuple: Enum2::Named21 {
                t: Enum1::Tuple11(vec![1, 2, 3, 4], vec![1, 2, 3, 4]),
            },
            named_named: Enum2::Named21 {
                t: Enum1::Named11 {
                    t: vec![1, 2, 3, 4],
                },
            },
        },
        &format!(
            "\
{}

{}

{}

{}

{}

{}
",
            Enum1::<Vec<usize>>::elm_definition().unwrap(),
            Enum1::<Vec<usize>>::encoder_definition().unwrap(),
            Enum1::<Vec<usize>>::decoder_definition().unwrap(),
            Enum2::<Enum1<Vec<usize>>>::elm_definition().unwrap(),
            Enum2::<Enum1<Vec<usize>>>::encoder_definition().unwrap(),
            Enum2::<Enum1<Vec<usize>>>::decoder_definition().unwrap(),
        ),
    )
}
