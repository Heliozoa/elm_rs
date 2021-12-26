use crate::{Elm, ElmJson};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
#[serde(transparent)]
struct TransparentNamed {
    field: u8,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
#[serde(transparent)]
struct TransparentNewtype(u8);

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
enum Other {
    A,
    #[serde(other)]
    B,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
#[serde(rename_all = "UPPERCASE")]
struct RenameStruct {
    uppercase: u8,
    #[serde(rename = "another-field")]
    renamed: u8,
    #[serde(rename(serialize = "se"))]
    rename_for_serialization: u8,
    #[serde(rename(deserialize = "de"))]
    rename_for_deserialization: u8,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
#[serde(rename_all = "UPPERCASE")]
enum RenameEnum {
    Uppercase,
    #[serde(rename = "another-variant")]
    Renamed,
    #[serde(rename(deserialize = "se"))]
    RenameForSerialization,
    #[serde(rename(serialize = "de"))]
    RenameForDeserialization,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmJson)]
struct Skip {
    #[serde(skip)]
    skipped: u8,
    not_skipped: u8,
}

#[test]
fn transparent_struct() {
    super::test(TransparentNamed { field: 0 });
}

#[test]
fn transparent_newtype() {
    super::test(TransparentNewtype(0));
}

#[test]
fn other() {
    let val: Other = super::test_with_json("\\\"other\\\"", "");
    assert_eq!(val, Other::B);
}

#[test]
fn rename_struct() {
    super::test(RenameStruct {
        uppercase: 0,
        renamed: 0,
        rename_for_serialization: 0,
        rename_for_deserialization: 0,
    })
}

#[test]
fn rename_enum() {
    super::test(RenameEnum::Uppercase);
    super::test(RenameEnum::Renamed);
    super::test(RenameEnum::RenameForSerialization);
    super::test(RenameEnum::RenameForDeserialization);
}

#[test]
fn skip() {
    super::test(Skip {
        skipped: 0,
        not_skipped: 0,
    });
}
