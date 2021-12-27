#![allow(dead_code)]

use jalava::{Elm, ElmForm, ElmFormParts, ElmJson};

#[derive(Elm, ElmJson, ElmFormParts)]
enum Filetype {
    Jpeg,
    Png,
}

#[derive(Elm, ElmJson, ElmForm)]
struct Drawing {
    title: String,
    authors: Vec<String>,
    filename: String,
    filetype: Filetype,
}

fn main() {
    // the target would typically be a file
    let mut target = vec![];
    // jalava provides a macro for conveniently creating an Elm module with everything needed
    jalava::export!("Bindings", &mut target, Drawing, Filetype; Drawing).unwrap();
    let output = String::from_utf8(target).unwrap();
    println!("{}", output);
}
