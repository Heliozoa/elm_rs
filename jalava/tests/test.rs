#![allow(dead_code)]

use jalava::Elm;

#[derive(Elm)]
struct Unit;

#[derive(Elm)]
struct Newtype(Unit);

#[derive(Elm)]
struct Tuple(Newtype, Newtype);

#[derive(Elm)]
struct Record<'a> {
    t1: (bool,),
    t2: (u32, f32),
    t3: (u32, u32, u32),
    b: Box<str>,
    p: Vec<&'a std::path::Path>,
    a: [u32; 123],
    arr: &'static [u32],
    o: Option<std::sync::Mutex<u32>>,
}

#[derive(Elm)]
enum CustomType<'a> {
    V1,
    V2(Newtype),
    V3(Tuple, Tuple),
    V4 { r: Record<'a> },
}

#[test]
fn test() {
    println!("{}", CustomType::encoder_definition().unwrap());
}
