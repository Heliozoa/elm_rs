#![allow(dead_code)]

use elm_rs::{Elm, ElmDecode, ElmEncode, ElmQuery, ElmQueryField};

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

#[derive(Elm, ElmQuery)]
struct Query {
    page: usize,
    thumbnail_size: Size,
}

#[derive(Elm, ElmQueryField)]
enum Size {
    Small,
    Large,
}

fn main() {
    // the target would typically be a file
    let mut target = vec![];
    // elm_rs provides a macro for conveniently creating an Elm module with everything needed
    elm_rs::export!("Bindings", &mut target, {
        encoders: [Filetype, Drawing], // generates Elm type definitions and encoders (requires ElmEncoder)
        decoders: [Filetype, Drawing], // generates Elm type definitions and decoders (requires ElmDecoder)
        queries: [Query],  // generates Elm type definitions and helper functions for forming queries (requires ElmQuery)
        query_fields: [Size],
    })
    .unwrap();
    let output = String::from_utf8(target).unwrap();
    println!("{}", output);
}
