#![allow(dead_code)]
use chrono::{DateTime, NaiveDate, NaiveDateTime, NaiveTime, Utc};
use jalava::{Elm, ElmJson};
use serde::Serialize;
use std::{
    borrow::Cow,
    cell::*,
    collections::*,
    num::*,
    path::*,
    rc::Rc,
    sync::{atomic::*, *},
    time::{Duration, SystemTime},
};
use uuid::Uuid;

fn assert_json<T: Serialize>(val: T, json: &str) {
    let ex: serde_json::Value = serde_json::from_str(json).unwrap();
    let ac = serde_json::to_value(&val).unwrap();
    assert_eq!(ex, ac);
}

#[derive(Serialize, Elm, ElmJson)]
struct Unit;

#[derive(Serialize, Elm, ElmJson)]
struct Newtype(bool);

#[derive(Serialize, Elm, ElmJson)]
struct Tuple(u32, Vec<Option<Vec<f64>>>);

#[derive(Serialize, Elm, ElmJson)]
struct Record<'a> {
    borrow: &'a bool,
    one_tuple: (bool,),
    two_tuple: (bool, bool),
    three_tuple: (bool, bool, bool),
    mut_borrow: &'a mut bool,
    arc: Arc<bool>,
    abool: AtomicBool,
    au8: AtomicU8,
    au16: AtomicU16,
    au32: AtomicU32,
    au64: AtomicU64,
    ausize: AtomicUsize,
    ai8: AtomicI8,
    ai16: AtomicI16,
    ai32: AtomicI32,
    ai64: AtomicI64,
    aisize: AtomicIsize,
    bmap: BTreeMap<String, bool>,
    bset: BTreeSet<bool>,
    b: Box<bool>,
    cell: Cell<bool>,
    cow: Cow<'a, str>,
    duration: Duration,
    map: HashMap<String, bool>,
    set: HashSet<bool>,
    list: LinkedList<bool>,
    mutex: Mutex<bool>,
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
    some: Option<bool>,
    none: Option<bool>,
    path: &'a Path,
    pathbuf: PathBuf,
    rc: Rc<bool>,
    refcell: RefCell<bool>,
    result_ok: Result<bool, bool>,
    result_err: Result<bool, bool>,
    rwlock: RwLock<bool>,
    string: String,
    system_time: SystemTime,
    vec: Vec<bool>,
    slice: &'a [bool],
    array: [bool; 2],
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
    pf32: f32,
    pf64: f64,
    ss: &'static str,
    uuid: Uuid,
    nt: NaiveTime,
    nd: NaiveDate,
    ndt: NaiveDateTime,
    dt: DateTime<Utc>,
}

#[derive(Serialize, Elm, ElmJson)]
enum CustomType<'a> {
    V1,
    V2(&'a Unit),
    V3(&'a Newtype, &'a Tuple),
    V4 { r: &'a Unit },
}

#[test]
fn json_test() {
    // check serde_json's output, the same strings should be used in the Elm test
    let unit = Unit;
    assert_json(&unit, "null");
    let newtype = Newtype(true);
    assert_json(&newtype, "true");
    let tuple = Tuple(123, vec![Some(vec![1.1, 2.2]), None]);
    assert_json(
        &tuple,
        r#"[
  123,
  [
    [
      1.1,
      2.2
    ],
    null
  ]
]"#,
    );

    let mut bmap = BTreeMap::new();
    bmap.insert("a".into(), true);
    let mut bset = BTreeSet::new();
    bset.insert(true);
    let mut map = HashMap::new();
    map.insert("b".into(), false);
    let mut set = HashSet::new();
    set.insert(false);
    let mut list = LinkedList::new();
    list.push_back(true);
    list.push_back(false);
    let record = Record {
        borrow: &true,
        mut_borrow: &mut false,
        one_tuple: (true,),
        two_tuple: (true, false),
        three_tuple: (true, false, true),
        arc: Arc::new(true),
        abool: AtomicBool::new(false),
        au8: AtomicU8::new(0),
        au16: AtomicU16::new(1),
        au32: AtomicU32::new(2),
        au64: AtomicU64::new(3),
        ausize: AtomicUsize::new(4),
        ai8: AtomicI8::new(5),
        ai16: AtomicI16::new(6),
        ai32: AtomicI32::new(7),
        ai64: AtomicI64::new(8),
        aisize: AtomicIsize::new(9),
        bmap,
        bset,
        b: Box::new(true),
        cell: Cell::new(false),
        cow: Cow::Borrowed("cow"),
        duration: Duration::from_nanos(1_000_000_123),
        map,
        set,
        list,
        mutex: Mutex::new(true),
        nu8: NonZeroU8::new(10).unwrap(),
        nu16: NonZeroU16::new(11).unwrap(),
        nu32: NonZeroU32::new(12).unwrap(),
        nu64: NonZeroU64::new(13).unwrap(),
        nu128: NonZeroU128::new(14).unwrap(),
        nusize: NonZeroUsize::new(15).unwrap(),
        ni8: NonZeroI8::new(16).unwrap(),
        ni16: NonZeroI16::new(17).unwrap(),
        ni32: NonZeroI32::new(18).unwrap(),
        ni64: NonZeroI64::new(19).unwrap(),
        ni128: NonZeroI128::new(20).unwrap(),
        nisize: NonZeroIsize::new(21).unwrap(),
        some: Some(false),
        none: None,
        path: &Path::new("path"),
        pathbuf: PathBuf::from("pathbuf"),
        rc: Rc::new(true),
        refcell: RefCell::new(false),
        result_ok: Ok(true),
        result_err: Err(false),
        rwlock: RwLock::new(true),
        string: "string".to_string(),
        system_time: SystemTime::UNIX_EPOCH,
        vec: vec![false, true],
        slice: &[false, true],
        array: [true, false],
        pu8: 22,
        pu16: 23,
        pu32: 24,
        pu64: 25,
        pu128: 26,
        pusize: 27,
        pi8: 28,
        pi16: 29,
        pi32: 30,
        pi64: 31,
        pi128: 32,
        pisize: 33,
        pf32: 34.349998474121094,
        pf64: 36.37,
        ss: "str",
        uuid: Uuid::parse_str("be81c148-3ebe-4e0b-949a-e4a706f4dbde").unwrap(),
        nt: NaiveTime::from_hms(11, 22, 33),
        nd: NaiveDate::from_ymd(2020, 10, 1),
        ndt: NaiveDateTime::from_timestamp(1629755240, 1234),
        dt: DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(1629755240, 1234), Utc),
    };
    assert_json(
        &record,
        r#"{
  "borrow": true,
  "one_tuple": [
    true
  ],
  "two_tuple": [
    true,
    false
  ],
  "three_tuple": [
    true,
    false,
    true
  ],
  "mut_borrow": false,
  "arc": true,
  "abool": false,
  "au8": 0,
  "au16": 1,
  "au32": 2,
  "au64": 3,
  "ausize": 4,
  "ai8": 5,
  "ai16": 6,
  "ai32": 7,
  "ai64": 8,
  "aisize": 9,
  "bmap": {
    "a": true
  },
  "bset": [
    true
  ],
  "b": true,
  "cell": false,
  "cow": "cow",
  "duration": {
    "secs": 1,
    "nanos": 123
  },
  "map": {
    "b": false
  },
  "set": [
    false
  ],
  "list": [
    true,
    false
  ],
  "mutex": true,
  "nu8": 10,
  "nu16": 11,
  "nu32": 12,
  "nu64": 13,
  "nu128": 14,
  "nusize": 15,
  "ni8": 16,
  "ni16": 17,
  "ni32": 18,
  "ni64": 19,
  "ni128": 20,
  "nisize": 21,
  "some": false,
  "none": null,
  "path": "path",
  "pathbuf": "pathbuf",
  "rc": true,
  "refcell": false,
  "result_ok": {
    "Ok": true
  },
  "result_err": {
    "Err": false
  },
  "rwlock": true,
  "string": "string",
  "system_time": {
    "secs_since_epoch": 0,
    "nanos_since_epoch": 0
  },
  "vec": [
    false,
    true
  ],
  "slice": [
    false,
    true
  ],
  "array": [
    true,
    false
  ],
  "pu8": 22,
  "pu16": 23,
  "pu32": 24,
  "pu64": 25,
  "pu128": 26,
  "pusize": 27,
  "pi8": 28,
  "pi16": 29,
  "pi32": 30,
  "pi64": 31,
  "pi128": 32,
  "pisize": 33,
  "pf32": 34.349998474121094,
  "pf64": 36.37,
  "ss": "str",
  "uuid": "be81c148-3ebe-4e0b-949a-e4a706f4dbde",
  "nt": "11:22:33",
  "nd": "2020-10-01",
  "ndt": "2021-08-23T21:47:20.000001234",
  "dt": "2021-08-23T21:47:20.000001234Z"
}"#,
    );

    assert_json(CustomType::V1, "\"V1\"");
    assert_json(
        CustomType::V2(&unit),
        r#"{
  "V2": null
}"#,
    );
    assert_json(
        CustomType::V3(&newtype, &tuple),
        r#"{
  "V3": [
    true,
    [
      123,
      [
        [
          1.1,
          2.2
        ],
        null
      ]
    ]
  ]
}"#,
    );
    assert_json(
        CustomType::V4 { r: &unit },
        r#"{
  "V4": {
    "r": null
  }
}"#,
    );
}

#[derive(Serialize, Elm, ElmJson)]
struct Generic<T: Elm + ElmJson>(T);

#[test]
fn generic() {
    let generic = Generic("hi");
    assert_json(&generic, "\"hi\"");
}

#[test]
fn elm_test() {
    // generate bindings
    let mut file = std::fs::File::create("./tests/elm/src/JsonBindings.elm").unwrap();
    jalava::export!(
    "JsonBindings", &mut file, Unit, Newtype, Tuple, Record, CustomType, Duration, SystemTime, Result<bool, bool>, Generic<&str>
  )
  .unwrap();

    // run elm-test
    let out = std::process::Command::new("elm-test")
        .current_dir("./tests/elm")
        .output()
        .unwrap();
    println!("{}", String::from_utf8_lossy(&out.stdout));
    println!("{}", String::from_utf8_lossy(&out.stderr));
    assert!(out.status.success());
}
