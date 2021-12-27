#![no_implicit_prelude]
#![allow(dead_code)]

#[derive(crate::jalava::Elm, crate::jalava::ElmJson, crate::jalava::ElmFormParts)]
enum Filetype {
    Jpeg,
    Png,
}

#[derive(crate::jalava::Elm, crate::jalava::ElmJson, crate::jalava::ElmForm)]
struct Drawing {
    filename: ::std::string::String,
    filetype: Filetype,
}

#[test]
fn hygiene() {
    let mut target = ::std::vec![];
    crate::jalava::export!("Bindings", &mut target, Drawing, Filetype; Drawing).unwrap();
}
