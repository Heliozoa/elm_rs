#![allow(dead_code)]

use elm_rs::{Elm, ElmDecode, ElmEncode};

#[derive(Elm, ElmEncode, ElmDecode)]
enum Filetype {
    Jpeg,
    Png,
}

#[derive(Elm, ElmEncode, ElmDecode)]
struct Drawing {
    title: String,
    authors: Vec<String>,
    filename: String,
    filetype: Filetype,
}

fn main() {
    // the target would typically be a file
    let mut target = vec![];
    // elm_rs provides a macro for conveniently creating an Elm module with everything needed
    elm_rs::export!("Bindings", &mut target, {
         both: [Filetype, Drawing], // generates both Elm encoders and decoders
         encoders: [], // generates only Elm encoders
         decoders: [], // you can leave any of these sections out if you don't have anything to put there
    })
    .unwrap();
    let output = String::from_utf8(target).unwrap();
    println!("{}", output);
}
