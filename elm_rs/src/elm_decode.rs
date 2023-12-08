//! Contains the `ElmDecode` trait.

use crate::Elm;
#[cfg(feature = "derive")]
pub use elm_rs_derive::ElmDecode;

/// Used to generate JSON decoders for our Rust types in Elm.
pub trait ElmDecode {
    /// The name of the decoder in Elm.
    fn decoder_type() -> String;
    /// The decoder function in Elm. None for decoders in Json.Decode.
    fn decoder_definition() -> Option<String>;
}

impl ElmDecode for () {
    fn decoder_type() -> String {
        "Json.Decode.null ()".to_string()
    }

    fn decoder_definition() -> Option<String> {
        None
    }
}

impl<T> ElmDecode for (T,)
where
    T: Elm + ElmDecode,
{
    fn decoder_type() -> String {
        ::std::format!("(Json.Decode.index 0 ({}))", T::decoder_type())
    }

    fn decoder_definition() -> Option<String> {
        None
    }
}

impl<T, U> ElmDecode for (T, U)
where
    T: Elm + ElmDecode,
    U: Elm + ElmDecode,
{
    fn decoder_type() -> String {
        ::std::format!(
            "Json.Decode.map2 (\\a b -> ( a, b )) (Json.Decode.index 0 ({})) (Json.Decode.index 1 ({}))",
            T::decoder_type(),
            U::decoder_type()
        )
    }

    fn decoder_definition() -> Option<String> {
        None
    }
}

impl<T, U, V> ElmDecode for (T, U, V)
where
    T: Elm + ElmDecode,
    U: Elm + ElmDecode,
    V: Elm + ElmDecode,
{
    fn decoder_type() -> String {
        ::std::format!(
            "Json.Decode.map3 (\\a b c -> ( a, b, c )) (Json.Decode.index 0 ({})) (Json.Decode.index 1 ({})) (Json.Decode.index 2 ({}))",
            T::decoder_type(),
            U::decoder_type(),
            V::decoder_type(),
        )
    }

    fn decoder_definition() -> Option<String> {
        None
    }
}

impl<T: Elm + ElmDecode + ToOwned + ?Sized> ElmDecode for std::borrow::Cow<'_, T> {
    fn decoder_type() -> String {
        T::decoder_type()
    }

    fn decoder_definition() -> Option<String> {
        T::decoder_definition()
    }
}

impl<T, const U: usize> ElmDecode for [T; U]
where
    T: Elm + ElmDecode,
{
    fn decoder_type() -> String {
        <[T]>::decoder_type()
    }

    fn decoder_definition() -> Option<String> {
        <[T]>::decoder_definition()
    }
}

impl ElmDecode for std::time::Duration {
    fn decoder_type() -> String {
        "durationDecoder".to_string()
    }

    fn decoder_definition() -> Option<String> {
        Some(
            r#"durationDecoder : Json.Decode.Decoder Duration
durationDecoder =
    Json.Decode.succeed Duration
    |> Json.Decode.andThen (\x -> Json.Decode.map x (Json.Decode.field "secs" (Json.Decode.int)))
    |> Json.Decode.andThen (\x -> Json.Decode.map x (Json.Decode.field "nanos" (Json.Decode.int)))
"#
            .to_string(),
        )
    }
}

impl<T: Elm + ElmDecode, E: Elm + ElmDecode> ElmDecode for Result<T, E> {
    fn decoder_type() -> String {
        format!(
            "resultDecoder ({}) ({})",
            E::decoder_type(),
            T::decoder_type()
        )
    }

    fn decoder_definition() -> Option<String> {
        Some(r#"resultDecoder : Json.Decode.Decoder e -> Json.Decode.Decoder t -> Json.Decode.Decoder (Result e t)
resultDecoder errDecoder okDecoder =
    Json.Decode.oneOf
        [ Json.Decode.map Ok (Json.Decode.field "Ok" okDecoder)
        , Json.Decode.map Err (Json.Decode.field "Err" errDecoder)
        ]"#
            .to_string())
    }
}

impl ElmDecode for std::time::SystemTime {
    fn decoder_type() -> String {
        "systemTimeDecoder".to_string()
    }

    fn decoder_definition() -> Option<String> {
        Some(
            r#"systemTimeDecoder : Json.Decode.Decoder SystemTime
systemTimeDecoder =
    Json.Decode.succeed SystemTime
    |> Json.Decode.andThen (\x -> Json.Decode.map x (Json.Decode.field "secs_since_epoch" (Json.Decode.int)))
    |> Json.Decode.andThen (\x -> Json.Decode.map x (Json.Decode.field "nanos_since_epoch" (Json.Decode.int)))
"#
            .to_string(),
        )
    }
}

macro_rules! impl_builtin {
    ($rust_type: ty, $elm_type: expr, $elm_decoder: expr) => {
        impl ElmDecode for $rust_type {
            fn decoder_type() -> String {
                $elm_decoder.to_string()
            }

            fn decoder_definition() -> Option<String> {
                None
            }
        }
    };
}

macro_rules! impl_builtin_container {
    ($rust_type: ty, $elm_name: expr, $elm_decoder: expr) => {
        impl<T: Elm + ElmDecode> ElmDecode for $rust_type {
            fn decoder_type() -> String {
                ::std::format!("{} ({})", $elm_decoder, T::decoder_type())
            }

            fn decoder_definition() -> Option<String> {
                None
            }
        }
    };
}

macro_rules! impl_builtin_map {
    ($rust_type: ty) => {
        impl<T: Elm + ElmDecode> ElmDecode for $rust_type {
            fn decoder_type() -> String {
                ::std::format!("Json.Decode.dict ({})", T::decoder_type())
            }

            fn decoder_definition() -> Option<String> {
                None
            }
        }
    };
}

macro_rules! impl_builtin_ptr {
    ($rust_type: ty) => {
        impl<T: Elm + ElmDecode + ?Sized> ElmDecode for $rust_type {
            fn decoder_type() -> String {
                ::std::format!("{}", T::decoder_type())
            }

            fn decoder_definition() -> Option<String> {
                T::decoder_definition()
            }
        }
    };
}

impl_builtin_ptr!(&'_ T);
impl_builtin_ptr!(&'_ mut T);
impl_builtin_ptr!(std::sync::Arc<T>);
impl_builtin!(std::sync::atomic::AtomicBool, "Bool", "Json.Decode.bool");
impl_builtin!(std::sync::atomic::AtomicU8, "Int", "Json.Decode.int");
impl_builtin!(std::sync::atomic::AtomicU16, "Int", "Json.Decode.int");
impl_builtin!(std::sync::atomic::AtomicU32, "Int", "Json.Decode.int");
impl_builtin!(std::sync::atomic::AtomicU64, "Int", "Json.Decode.int");
impl_builtin!(std::sync::atomic::AtomicUsize, "Int", "Json.Decode.int");
impl_builtin!(std::sync::atomic::AtomicI8, "Int", "Json.Decode.int");
impl_builtin!(std::sync::atomic::AtomicI16, "Int", "Json.Decode.int");
impl_builtin!(std::sync::atomic::AtomicI32, "Int", "Json.Decode.int");
impl_builtin!(std::sync::atomic::AtomicI64, "Int", "Json.Decode.int");
impl_builtin!(std::sync::atomic::AtomicIsize, "Int", "Json.Decode.int");
impl_builtin_map!(std::collections::BTreeMap<String,T>);
impl_builtin_container!(std::collections::BTreeSet<T>, "List", "Json.Decode.list");
impl_builtin_ptr!(Box<T>);
impl_builtin_ptr!(std::cell::Cell<T>);
impl_builtin_map!(std::collections::HashMap<String,T>);
impl_builtin_container!(std::collections::HashSet<T>, "List", "Json.Decode.list");
// todo ipaddrs
impl_builtin_container!(std::collections::LinkedList<T>, "List", "Json.Decode.list");
impl_builtin_ptr!(std::sync::Mutex<T>);
impl_builtin!(std::num::NonZeroU8, "Int", "Json.Decode.int");
impl_builtin!(std::num::NonZeroU16, "Int", "Json.Decode.int");
impl_builtin!(std::num::NonZeroU32, "Int", "Json.Decode.int");
impl_builtin!(std::num::NonZeroU64, "Int", "Json.Decode.int");
impl_builtin!(std::num::NonZeroU128, "Int", "Json.Decode.int");
impl_builtin!(std::num::NonZeroUsize, "Int", "Json.Decode.int");
impl_builtin!(std::num::NonZeroI8, "Int", "Json.Decode.int");
impl_builtin!(std::num::NonZeroI16, "Int", "Json.Decode.int");
impl_builtin!(std::num::NonZeroI32, "Int", "Json.Decode.int");
impl_builtin!(std::num::NonZeroI64, "Int", "Json.Decode.int");
impl_builtin!(std::num::NonZeroI128, "Int", "Json.Decode.int");
impl_builtin!(std::num::NonZeroIsize, "Int", "Json.Decode.int");
impl_builtin_container!(Option<T>, "Maybe", "Json.Decode.nullable");
impl_builtin!(std::path::Path, "String", "Json.Decode.string");
impl_builtin!(std::path::PathBuf, "String", "Json.Decode.string");
// todo phantomdata
impl_builtin_ptr!(std::rc::Rc<T>);
impl_builtin_ptr!(std::cell::RefCell<T>);
impl_builtin_ptr!(std::sync::RwLock<T>);
// todo socketaddrs
impl_builtin!(String, "String", "Json.Decode.string");
impl_builtin_container!(Vec<T>, "List", "Json.Decode.list");
impl_builtin_container!([T], "List", "Json.Decode.list");
impl_builtin!(bool, "Bool", "Json.Decode.bool");
impl_builtin!(u8, "Int", "Json.Decode.int");
impl_builtin!(u16, "Int", "Json.Decode.int");
impl_builtin!(u32, "Int", "Json.Decode.int");
impl_builtin!(u64, "Int", "Json.Decode.int");
impl_builtin!(u128, "Int", "Json.Decode.int");
impl_builtin!(usize, "Int", "Json.Decode.int");
impl_builtin!(i8, "Int", "Json.Decode.int");
impl_builtin!(i16, "Int", "Json.Decode.int");
impl_builtin!(i32, "Int", "Json.Decode.int");
impl_builtin!(i64, "Int", "Json.Decode.int");
impl_builtin!(i128, "Int", "Json.Decode.int");
impl_builtin!(isize, "Int", "Json.Decode.int");
impl_builtin!(f32, "Float", "Json.Decode.float");
impl_builtin!(f64, "Float", "Json.Decode.float");
impl_builtin!(str, "String", "Json.Decode.string");

#[cfg(feature = "uuid")]
impl_builtin!(uuid::Uuid, "String", "Json.Decode.string");

#[cfg(feature = "chrono")]
impl_builtin!(chrono::NaiveTime, "String", "Json.Decode.string");
#[cfg(feature = "chrono")]
impl_builtin!(chrono::NaiveDate, "String", "Json.Decode.string");
#[cfg(feature = "chrono")]
impl_builtin!(chrono::NaiveDateTime, "String", "Json.Decode.string");
#[cfg(feature = "chrono")]
impl<T: chrono::TimeZone> ElmDecode for chrono::DateTime<T> {
    fn decoder_type() -> String {
        String::decoder_type()
    }

    fn decoder_definition() -> Option<String> {
        String::decoder_definition()
    }
}
