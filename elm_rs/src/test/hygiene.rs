#![no_implicit_prelude]
#![allow(dead_code)]

#[derive(crate::elm_rs::Elm, crate::elm_rs::ElmEncode, crate::elm_rs::ElmDecode)]
enum Filetype {
    Jpeg,
    Png,
}

#[derive(crate::elm_rs::Elm, crate::elm_rs::ElmEncode, crate::elm_rs::ElmDecode)]
struct Drawing {
    filename: ::std::string::String,
    filetype: Filetype,
}

#[test]
fn hygiene() {
    let mut target = ::std::vec![];
    crate::elm_rs::export!("Bindings", &mut target, Drawing, Filetype).unwrap();
}
