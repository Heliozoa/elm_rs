use crate::{Elm, ElmJson};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
enum E {
    Unit,
    Newtype(u8),
    Tuple(u8, u8),
    Named { u8: u8 },
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
struct S {
    e1: Vec<E>,
    e2: Vec<E>,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
enum E2 {
    E2e(Vec<E>),
    E2s(Vec<S>),
    E2es(Vec<E>, Vec<S>),
    E2se { es: Vec<S>, ee: Vec<E> },
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
struct S2 {
    fe: Vec<E>,
    fs: Vec<S>,
    fe2: Vec<E2>,
}

#[test]
fn complex() {
    super::test_with_deps(
        S2 {
            fe: vec![E::Unit, E::Newtype(0), E::Tuple(0, 0), E::Named { u8: 0 }],
            fs: vec![S {
                e1: vec![E::Unit, E::Newtype(0), E::Tuple(0, 0), E::Named { u8: 0 }],
                e2: vec![E::Unit, E::Newtype(0), E::Tuple(0, 0), E::Named { u8: 0 }],
            }],
            fe2: vec![E2::E2e(vec![
                E::Unit,
                E::Newtype(0),
                E::Tuple(0, 0),
                E::Named { u8: 0 },
            ])],
        },
        &::std::format!(
            "\
{}

{}

{}

{}

{}

{}

{}

{}

{}

",
            E::elm_definition().unwrap(),
            S::elm_definition().unwrap(),
            E2::elm_definition().unwrap(),
            E::encoder_definition().unwrap(),
            S::encoder_definition().unwrap(),
            E2::encoder_definition().unwrap(),
            E::decoder_definition().unwrap(),
            S::decoder_definition().unwrap(),
            E2::decoder_definition().unwrap(),
        ),
    )
}
