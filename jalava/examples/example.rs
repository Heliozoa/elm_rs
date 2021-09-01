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
    let elm_types = format!(
        "
{}

{}

{}

{}

{}

{}

{}

{}",
        Filetype::elm_definition().unwrap(),
        Filetype::decoder_definition().unwrap(),
        Filetype::encoder_definition().unwrap(),
        Filetype::to_string_definition().unwrap(),
        Drawing::elm_definition().unwrap(),
        Drawing::decoder_definition().unwrap(),
        Drawing::encoder_definition().unwrap(),
        Drawing::prepare_form(),
    );
    println!("{}", elm_types);
}
