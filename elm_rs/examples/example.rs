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
        // generates types and encoders for types implementing ElmEncoder
        encoders: [Filetype, Drawing],
        // generates types and decoders for types implementing ElmDecoder
        decoders: [Filetype, Drawing],
        // generates types and functions for forming queries for types implementing ElmQuery
        queries: [Query],
        // generates types and functions for forming queries for types implementing ElmQueryField
        query_fields: [Size],
    })
    .unwrap();
    let output = String::from_utf8(target).unwrap();
    println!("{}", output);
}
