use crate::{Elm, ElmDecode, ElmEncode};
use serde::{Deserialize, Serialize};

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmEncode, ElmDecode)]
#[serde(transparent)]
struct TransparentNamed {
    field: u8,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmEncode, ElmDecode)]
#[serde(transparent)]
struct TransparentNewtype(u8);

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmEncode, ElmDecode)]
enum Other {
    A,
    #[serde(other)]
    B,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmEncode, ElmDecode)]
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

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmEncode, ElmDecode)]
#[serde(rename_all = "UPPERCASE")]
enum RenameEnum {
    Uppercase,
    #[serde(rename = "another-variant")]
    Renamed,
    #[serde(rename(serialize = "se"))]
    RenameForSerialization,
    #[serde(rename(deserialize = "de"))]
    RenameForDeserialization,
}

#[derive(Debug, PartialEq, Deserialize, Serialize, Elm, ElmEncode, ElmDecode)]
struct Skip {
    #[serde(skip)]
    skipped: u8,
    not_skipped: u8,
}

#[test]
fn transparent_struct() {
    super::test_json(TransparentNamed { field: 0 });
}

#[test]
fn transparent_newtype() {
    super::test_json(TransparentNewtype(0));
}

#[test]
fn other() {
    let val: Other = super::test_with_json("\\\"other\\\"", "");
    assert_eq!(val, Other::B);
}

#[test]
fn rename_struct() {
    super::test_json(RenameStruct {
        uppercase: 0,
        renamed: 0,
        rename_for_serialization: 0,
        rename_for_deserialization: 0,
    })
}

#[test]
fn rename_enum() {
    super::test_json(RenameEnum::Uppercase);
    super::test_json(RenameEnum::Renamed);
    super::test_json(RenameEnum::RenameForSerialization);
    super::test_json(RenameEnum::RenameForDeserialization);
}

#[test]
fn skip() {
    super::test_json(Skip {
        skipped: 0,
        not_skipped: 0,
    });
}
