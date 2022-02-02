//! Contains the `ElmJson` trait.

use crate::Elm;

#[cfg(feature = "derive")]
pub use elm_rs_derive::ElmJson;

/// Used to generate JSON encoders and decoders for our Rust types in Elm.
pub trait ElmJson {
    /// The name of the decoder in Elm.
    fn decoder_type() -> String;
    /// The decoder function in Elm. None for decoders in Json.Decode.
    fn decoder_definition() -> Option<String>;
    /// The name of the encoder in Elm.
    fn encoder_type() -> String;
    /// The encoder function in Elm. None for encoders in Json.Encode.
    fn encoder_definition() -> Option<String>;
}

impl ElmJson for () {
    fn decoder_type() -> String {
        "Json.Decode.null ()".to_string()
    }

    fn decoder_definition() -> Option<String> {
        None
    }

    fn encoder_type() -> String {
        r#"(\_ -> Json.Encode.null)"#.to_string()
    }

    fn encoder_definition() -> Option<String> {
        None
    }
}

impl<T> ElmJson for (T,)
where
    T: Elm + ElmJson,
{
    fn decoder_type() -> String {
        ::std::format!("(Json.Decode.index 0 ({}))", T::decoder_type(),)
    }

    fn decoder_definition() -> Option<String> {
        None
    }

    fn encoder_type() -> String {
        ::std::format!(
            "\\( a ) -> Json.Encode.list identity [ {} a ]",
            T::encoder_type(),
        )
    }

    fn encoder_definition() -> Option<String> {
        None
    }
}

impl<T, U> ElmJson for (T, U)
where
    T: Elm + ElmJson,
    U: Elm + ElmJson,
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

    fn encoder_type() -> String {
        ::std::format!(
            "\\( a, b) -> Json.Encode.list identity [ {} a, {} b ]",
            T::encoder_type(),
            U::encoder_type(),
        )
    }

    fn encoder_definition() -> Option<String> {
        None
    }
}

impl<T, U, V> ElmJson for (T, U, V)
where
    T: Elm + ElmJson,
    U: Elm + ElmJson,
    V: Elm + ElmJson,
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

    fn encoder_type() -> String {
        ::std::format!(
            "\\( a, b, c ) -> Json.Encode.list identity [ {} a, {} b, {} c ]",
            T::encoder_type(),
            U::encoder_type(),
            V::encoder_type(),
        )
    }

    fn encoder_definition() -> Option<String> {
        None
    }
}

impl<T: Elm + ElmJson + ToOwned + ?Sized> ElmJson for std::borrow::Cow<'_, T> {
    fn decoder_type() -> String {
        T::decoder_type()
    }

    fn decoder_definition() -> Option<String> {
        T::decoder_definition()
    }

    fn encoder_type() -> String {
        T::encoder_type()
    }

    fn encoder_definition() -> Option<String> {
        T::encoder_definition()
    }
}

impl<T, const U: usize> ElmJson for [T; U]
where
    T: Elm + ElmJson,
{
    fn decoder_type() -> String {
        <[T]>::decoder_type()
    }

    fn decoder_definition() -> Option<String> {
        <[T]>::decoder_definition()
    }

    fn encoder_type() -> String {
        <[T]>::encoder_type()
    }

    fn encoder_definition() -> Option<String> {
        <[T]>::encoder_definition()
    }
}

impl ElmJson for std::time::Duration {
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

    fn encoder_type() -> String {
        "durationEncoder".to_string()
    }

    fn encoder_definition() -> Option<String> {
        Some(
            r#"durationEncoder : Duration -> Json.Encode.Value
durationEncoder duration =
    Json.Encode.object
    [ ( "secs", Json.Encode.int duration.secs )
    , ( "nanos", Json.Encode.int duration.nanos )
    ]
"#
            .to_string(),
        )
    }
}

impl<T: Elm + ElmJson, E: Elm + ElmJson> ElmJson for Result<T, E> {
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

    fn encoder_type() -> String {
        format!(
            "resultEncoder ({}) ({})",
            E::encoder_type(),
            T::encoder_type()
        )
    }

    fn encoder_definition() -> Option<String> {
        Some(r#"resultEncoder : (e -> Json.Encode.Value) -> (t -> Json.Encode.Value) -> (Result e t -> Json.Encode.Value)
resultEncoder errEncoder okEncoder enum =
    case enum of
        Ok inner ->
            Json.Encode.object [ ( "Ok", okEncoder inner ) ]
        Err inner ->
            Json.Encode.object [ ( "Err", errEncoder inner ) ]"#
            .to_string())
    }
}

impl ElmJson for std::time::SystemTime {
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

    fn encoder_type() -> String {
        "systemTimeEncoder".to_string()
    }

    fn encoder_definition() -> Option<String> {
        Some(
            r#"systemTimeEncoder : SystemTime -> Json.Encode.Value
systemTimeEncoder duration =
    Json.Encode.object
    [ ( "secs_since_epoch", Json.Encode.int duration.secs_since_epoch )
    , ( "nanos_since_epoch", Json.Encode.int duration.nanos_since_epoch )
    ]
"#
            .to_string(),
        )
    }
}

macro_rules! impl_builtin {
    ($rust_type: ty, $elm_type: expr, $elm_decoder: expr, $elm_encoder: expr) => {
        impl ElmJson for $rust_type {
            fn decoder_type() -> String {
                $elm_decoder.to_string()
            }

            fn decoder_definition() -> Option<String> {
                None
            }

            fn encoder_type() -> String {
                $elm_encoder.to_string()
            }

            fn encoder_definition() -> Option<String> {
                None
            }
        }
    };
}

macro_rules! impl_builtin_container {
    ($rust_type: ty, $elm_name: expr, $elm_decoder: expr, $elm_encoder: expr) => {
        impl<T: Elm + ElmJson> ElmJson for $rust_type {
            fn decoder_type() -> String {
                ::std::format!("{} ({})", $elm_decoder, T::decoder_type())
            }

            fn decoder_definition() -> Option<String> {
                None
            }

            fn encoder_type() -> String {
                ::std::format!("{} ({})", $elm_encoder, T::encoder_type())
            }

            fn encoder_definition() -> Option<String> {
                None
            }
        }
    };
}

macro_rules! impl_builtin_map {
    ($rust_type: ty) => {
        impl<T: Elm + ElmJson> ElmJson for $rust_type {
            fn decoder_type() -> String {
                ::std::format!("Json.Decode.dict ({})", T::decoder_type())
            }

            fn decoder_definition() -> Option<String> {
                None
            }

            fn encoder_type() -> String {
                ::std::format!("Json.Encode.dict identity ({})", T::encoder_type())
            }

            fn encoder_definition() -> Option<String> {
                None
            }
        }
    };
}

macro_rules! impl_builtin_ptr {
    ($rust_type: ty) => {
        impl<T: Elm + ElmJson + ?Sized> ElmJson for $rust_type {
            fn decoder_type() -> String {
                ::std::format!("{}", T::decoder_type())
            }

            fn decoder_definition() -> Option<String> {
                T::decoder_definition()
            }

            fn encoder_type() -> String {
                ::std::format!("{}", T::encoder_type())
            }

            fn encoder_definition() -> Option<String> {
                T::encoder_definition()
            }
        }
    };
}

impl_builtin_ptr!(&'_ T);
impl_builtin_ptr!(&'_ mut T);
impl_builtin_ptr!(std::sync::Arc<T>);
impl_builtin!(
    std::sync::atomic::AtomicBool,
    "Bool",
    "Json.Decode.bool",
    "Json.Encode.bool"
);
impl_builtin!(
    std::sync::atomic::AtomicU8,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::sync::atomic::AtomicU16,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::sync::atomic::AtomicU32,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::sync::atomic::AtomicU64,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::sync::atomic::AtomicUsize,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::sync::atomic::AtomicI8,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::sync::atomic::AtomicI16,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::sync::atomic::AtomicI32,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::sync::atomic::AtomicI64,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::sync::atomic::AtomicIsize,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin_map!(std::collections::BTreeMap<String,T>);
impl_builtin_container!(
    std::collections::BTreeSet<T>,
    "List",
    "Json.Decode.list",
    "Json.Encode.list"
);
impl_builtin_ptr!(Box<T>);
impl_builtin_ptr!(std::cell::Cell<T>);
impl_builtin_map!(std::collections::HashMap<String,T>);
impl_builtin_container!(
    std::collections::HashSet<T>,
    "List",
    "Json.Decode.list",
    "Json.Encode.list"
);
// todo ipaddrs
impl_builtin_container!(
    std::collections::LinkedList<T>,
    "List",
    "Json.Decode.list",
    "Json.Encode.list"
);
impl_builtin_ptr!(std::sync::Mutex<T>);
impl_builtin!(
    std::num::NonZeroU8,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::num::NonZeroU16,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::num::NonZeroU32,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::num::NonZeroU64,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::num::NonZeroU128,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::num::NonZeroUsize,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::num::NonZeroI8,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::num::NonZeroI16,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::num::NonZeroI32,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::num::NonZeroI64,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::num::NonZeroI128,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin!(
    std::num::NonZeroIsize,
    "Int",
    "Json.Decode.int",
    "Json.Encode.int"
);
impl_builtin_container!(
    Option<T>,
    "Maybe",
    "Json.Decode.nullable",
    "Maybe.withDefault Json.Encode.null << Maybe.map"
);
impl_builtin!(
    std::path::Path,
    "String",
    "Json.Decode.string",
    "Json.Encode.string"
);
impl_builtin!(
    std::path::PathBuf,
    "String",
    "Json.Decode.string",
    "Json.Encode.string"
);
// todo phantomdata
impl_builtin_ptr!(std::rc::Rc<T>);
impl_builtin_ptr!(std::cell::RefCell<T>);
impl_builtin_ptr!(std::sync::RwLock<T>);
// todo socketaddrs
impl_builtin!(String, "String", "Json.Decode.string", "Json.Encode.string");
impl_builtin_container!(Vec<T>, "List", "Json.Decode.list", "Json.Encode.list");
impl_builtin_container!([T], "List", "Json.Decode.list", "Json.Encode.list");
impl_builtin!(bool, "Bool", "Json.Decode.bool", "Json.Encode.bool");
impl_builtin!(u8, "Int", "Json.Decode.int", "Json.Encode.int");
impl_builtin!(u16, "Int", "Json.Decode.int", "Json.Encode.int");
impl_builtin!(u32, "Int", "Json.Decode.int", "Json.Encode.int");
impl_builtin!(u64, "Int", "Json.Decode.int", "Json.Encode.int");
impl_builtin!(u128, "Int", "Json.Decode.int", "Json.Encode.int");
impl_builtin!(usize, "Int", "Json.Decode.int", "Json.Encode.int");
impl_builtin!(i8, "Int", "Json.Decode.int", "Json.Encode.int");
impl_builtin!(i16, "Int", "Json.Decode.int", "Json.Encode.int");
impl_builtin!(i32, "Int", "Json.Decode.int", "Json.Encode.int");
impl_builtin!(i64, "Int", "Json.Decode.int", "Json.Encode.int");
impl_builtin!(i128, "Int", "Json.Decode.int", "Json.Encode.int");
impl_builtin!(isize, "Int", "Json.Decode.int", "Json.Encode.int");
impl_builtin!(f32, "Float", "Json.Decode.float", "Json.Encode.float");
impl_builtin!(f64, "Float", "Json.Decode.float", "Json.Encode.float");
impl_builtin!(str, "String", "Json.Decode.string", "Json.Encode.string");

#[cfg(feature = "uuid")]
impl_builtin!(
    uuid::Uuid,
    "String",
    "Json.Decode.string",
    "Json.Encode.string"
);

#[cfg(feature = "chrono")]
impl_builtin!(
    chrono::NaiveTime,
    "String",
    "Json.Decode.string",
    "Json.Encode.string"
);
#[cfg(feature = "chrono")]
impl_builtin!(
    chrono::NaiveDate,
    "String",
    "Json.Decode.string",
    "Json.Encode.string"
);
#[cfg(feature = "chrono")]
impl_builtin!(
    chrono::NaiveDateTime,
    "String",
    "Json.Decode.string",
    "Json.Encode.string"
);
#[cfg(feature = "chrono")]
impl<T: chrono::TimeZone> ElmJson for chrono::DateTime<T> {
    fn decoder_type() -> String {
        String::decoder_type()
    }

    fn decoder_definition() -> Option<String> {
        String::decoder_definition()
    }

    fn encoder_type() -> String {
        String::encoder_type()
    }

    fn encoder_definition() -> Option<String> {
        String::encoder_definition()
    }
}
