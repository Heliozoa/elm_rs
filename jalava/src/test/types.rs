use std::{
    borrow::Cow,
    cell::{Cell, RefCell},
    collections::*,
    hash::Hash,
    num::*,
    path::PathBuf,
    rc::Rc,
    sync::{atomic::*, Arc, Mutex, RwLock},
    time::{Duration, SystemTime},
};

use crate::{Elm, ElmJson};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Elm, ElmJson)]
struct Types<T: Copy + Hash + Eq> {
    t: T,
    one: (u8,),
    two: (u8, u8),
    three: (u8, u8, u8),
    arc: Arc<T>,
    abool: AtomicBool,
    ai8: AtomicI8,
    ai16: AtomicI16,
    ai32: AtomicI32,
    ai64: AtomicI64,
    aisize: AtomicIsize,
    au8: AtomicU8,
    au16: AtomicU16,
    au32: AtomicU32,
    au64: AtomicU64,
    ausize: AtomicUsize,
    btreemap: BTreeMap<String, T>,
    btreeset: BTreeSet<u8>,
    b: Box<T>,
    cell: Cell<T>,
    cow: Cow<'static, u8>,
    duration: Duration,
    hashmap: HashMap<String, T>,
    hashset: HashSet<T>,
    linkedlist: LinkedList<T>,
    mutex: Mutex<T>,
    nu8: NonZeroU8,
    nu16: NonZeroU16,
    nu32: NonZeroU32,
    nu64: NonZeroU64,
    nusize: NonZeroUsize,
    ni8: NonZeroI8,
    ni16: NonZeroI16,
    ni32: NonZeroI32,
    ni64: NonZeroI64,
    nisize: NonZeroIsize,
    option: Option<T>,
    pathbuf: PathBuf,
    rc: Rc<T>,
    refcell: RefCell<T>,
    result: Result<T, T>,
    rwlock: RwLock<T>,
    string: String,
    systemtime: SystemTime,
    vec: Vec<T>,
    array: [T; 2],
    bool: bool,
    f32: f32,
    f64: f64,
    u8: u8,
    u16: u16,
    u32: u32,
    u64: u64,
    usize: usize,
    i8: i8,
    i16: i16,
    i32: i32,
    i64: i64,
    isize: isize,
}

#[test]
fn types() {
    super::test_without_eq(
        &Types {
            t: 0u8,
            one: (0,),
            two: (0, 0),
            three: (0, 0, 0),
            arc: Arc::new(0),
            abool: AtomicBool::default(),
            ai8: AtomicI8::default(),
            ai16: AtomicI16::default(),
            ai32: AtomicI32::default(),
            ai64: AtomicI64::default(),
            aisize: AtomicIsize::default(),
            au8: AtomicU8::default(),
            au16: AtomicU16::default(),
            au32: AtomicU32::default(),
            au64: AtomicU64::default(),
            ausize: AtomicUsize::default(),
            btreemap: BTreeMap::default(),
            btreeset: BTreeSet::default(),
            b: Box::new(0),
            cell: Cell::new(0),
            cow: Cow::Owned(0),
            duration: Duration::from_secs(0),
            hashmap: HashMap::default(),
            hashset: HashSet::default(),
            linkedlist: LinkedList::default(),
            mutex: Mutex::new(0),
            nu8: NonZeroU8::new(1).unwrap(),
            nu16: NonZeroU16::new(1).unwrap(),
            nu32: NonZeroU32::new(1).unwrap(),
            nu64: NonZeroU64::new(1).unwrap(),
            nusize: NonZeroUsize::new(1).unwrap(),
            ni8: NonZeroI8::new(1).unwrap(),
            ni16: NonZeroI16::new(1).unwrap(),
            ni32: NonZeroI32::new(1).unwrap(),
            ni64: NonZeroI64::new(1).unwrap(),
            nisize: NonZeroIsize::new(1).unwrap(),
            option: None,
            pathbuf: PathBuf::default(),
            rc: Rc::new(0),
            refcell: RefCell::new(0),
            result: Err(0),
            rwlock: RwLock::new(0),
            string: "0".to_string(),
            systemtime: SystemTime::UNIX_EPOCH,
            vec: vec![0, 0],
            array: [0, 0],
            bool: false,
            f32: 0.0,
            f64: 0.0,
            u8: 0,
            u16: 0,
            u32: 0,
            u64: 0,
            usize: 0,
            i8: 0,
            i16: 0,
            i32: 0,
            i64: 0,
            isize: 0,
        },
        &::std::format!(
            "\
{}

{}

{}

{}

{}

{}

{}

{}

",
            std::time::Duration::elm_definition().unwrap(),
            std::time::Duration::encoder_definition().unwrap(),
            std::time::Duration::decoder_definition().unwrap(),
            std::time::SystemTime::elm_definition().unwrap(),
            std::time::SystemTime::encoder_definition().unwrap(),
            std::time::SystemTime::decoder_definition().unwrap(),
            Result::<u8, u8>::encoder_definition().unwrap(),
            Result::<u8, u8>::decoder_definition().unwrap(),
        ),
    );
}
