#![allow(dead_code)]

use jalava::{Elm, ElmJson};

#[derive(Elm, ElmJson)]
enum Filetype {
    Jpeg,
    Png,
}

#[derive(Elm, ElmJson)]
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
    jalava::export!("Bindings", &mut target, Drawing, Filetype).unwrap();
    let output = String::from_utf8(target).unwrap();
    println!("{}", output);
}
