#![allow(dead_code)]
#![allow(unused_variables)]

use jalava::{Elm, ElmForm};
use rocket::{config::Shutdown, form::Form, log::LogLevel, Config, Request};
use std::{
    collections::HashMap,
    num::*,
    process::Command,
    sync::atomic::{AtomicBool, Ordering},
    time::Duration,
};
use tokio::time::sleep;

#[macro_use]
extern crate rocket;

static SUCCESS_A: AtomicBool = AtomicBool::new(false);
static SUCCESS_B: AtomicBool = AtomicBool::new(false);
static SUCCESS_C: AtomicBool = AtomicBool::new(false);
static SUCCESS_D: AtomicBool = AtomicBool::new(false);
static ERROR: AtomicBool = AtomicBool::new(false);

#[tokio::test]
#[ignore = "requires manual action"]
async fn rocket_test() {
    let config = Config {
        port: 8001,
        log_level: LogLevel::Off,
        shutdown: Shutdown {
            ctrlc: false,
            ..Default::default()
        },
        ..Config::debug_default()
    };
    tokio::spawn(
        rocket::build()
            .configure(config)
            .mount("/", routes![a, b, c, d])
            .register("/", catchers![err])
            .launch(),
    );

    let mut file = std::fs::File::create("./tests/elm/src/FormBindings.elm").unwrap();
    jalava::export!("FormBindings", &mut file; A, B, C, D).unwrap();
    let mut child = Command::new("elm")
        .arg("reactor")
        .current_dir("./tests/elm/")
        .spawn()
        .unwrap();

    loop {
        println!("checking status, visit http://localhost:8000/src/Main.elm to send form requests that the tests are expecting");
        if ERROR.load(Ordering::Relaxed) {
            panic!();
        }
        if SUCCESS_A.load(Ordering::Relaxed)
            && SUCCESS_B.load(Ordering::Relaxed)
            && SUCCESS_C.load(Ordering::Relaxed)
            && SUCCESS_D.load(Ordering::Relaxed)
        {
            break;
        }
        sleep(Duration::from_secs(1)).await;
    }

    child.kill().unwrap();
}

#[derive(FromForm, Elm, ElmForm)]
struct A<'a> {
    string: String,
    s: &'a str,
    b: bool,
    pu8: u8,
    pu16: u16,
    pu32: u32,
    pu64: u64,
    pu128: u128,
    pusize: usize,
    pi8: i8,
    pi16: i16,
    pi32: i32,
    pi64: i64,
    pi128: i128,
    pisize: isize,
    nu8: NonZeroU8,
    nu16: NonZeroU16,
    nu32: NonZeroU32,
    nu64: NonZeroU64,
    nu128: NonZeroU128,
    nusize: NonZeroUsize,
    ni8: NonZeroI8,
    ni16: NonZeroI16,
    ni32: NonZeroI32,
    ni64: NonZeroI64,
    ni128: NonZeroI128,
    nisize: NonZeroIsize,
    pf32: f32,
    pf64: f64,
}

#[post("/a", data = "<form>")]
async fn a(form: Form<A<'_>>) {
    SUCCESS_A.store(true, Ordering::Relaxed);
}

#[derive(FromForm, Elm, ElmForm)]
struct B {
    s: String,
    ss: Vec<String>,
    sm: HashMap<String, Vec<String>>,
}

#[post("/b", data = "<form>")]
async fn b(form: Form<B>) {
    SUCCESS_B.store(true, Ordering::Relaxed);
}

#[derive(FromForm, Elm, ElmForm)]
struct C {
    b: B,
    bs: Vec<B>,
    bm: HashMap<String, Vec<B>>,
}

#[post("/c", data = "<form>")]
async fn c(form: Form<C>) {
    SUCCESS_C.store(true, Ordering::Relaxed);
}

#[derive(FromForm, Elm, ElmForm)]
struct D {
    c: C,
}

#[post("/d", data = "<form>")]
async fn d(form: Form<D>) {
    SUCCESS_D.store(true, Ordering::Relaxed);
}

#[catch(default)]
fn err(_: &Request) {
    ERROR.store(true, Ordering::Relaxed);
}
