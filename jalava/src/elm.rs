pub use jalava_derive::Elm;

pub trait Elm {
    /// The name of the type in Elm.
    fn elm_type() -> String;
    /// The definition of the type in Elm. None for types in Elm's std.
    fn elm_definition() -> Option<String>;
    /// The name of the decoder in Elm.
    fn decoder_type() -> String;
    /// The decoder function in Elm. None for decoders in Json.Decode.
    fn decoder_definition() -> Option<String>;
    /// The name of the encoder in Elm.
    fn encoder_type() -> String;
    /// The encoder function in Elm. None for encoders in Json.Encode.
    fn encoder_definition() -> Option<String>;
}

impl<T> Elm for (T,)
where
    T: Elm,
{
    fn elm_type() -> String {
        T::elm_type()
    }

    fn elm_definition() -> Option<String> {
        T::elm_definition()
    }

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

impl<T, U> Elm for (T, U)
where
    T: Elm,
    U: Elm,
{
    fn elm_type() -> String {
        format!("( {}, {} )", T::elm_type(), U::elm_type())
    }

    fn elm_definition() -> Option<String> {
        None
    }

    fn decoder_type() -> String {
        format!(
            "Json.Decode.map2 (\\a b -> ( a, b )) (Json.Decode.index 0 ({})) (Json.Decode.index 1 ({}))",
            T::decoder_type(),
            U::decoder_type()
        )
    }

    fn decoder_definition() -> Option<String> {
        None
    }

    fn encoder_type() -> String {
        format!(
            "\\( a, b) -> Json.Encode.list identity [ {} a, {} b ]",
            T::encoder_type(),
            U::encoder_type(),
        )
    }

    fn encoder_definition() -> Option<String> {
        None
    }
}

impl<T, U, V> Elm for (T, U, V)
where
    T: Elm,
    U: Elm,
    V: Elm,
{
    fn elm_type() -> String {
        format!(
            "( {}, {}, {} )",
            T::elm_type(),
            U::elm_type(),
            V::elm_type()
        )
    }

    fn elm_definition() -> Option<String> {
        None
    }

    fn decoder_type() -> String {
        format!(
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
        format!(
            "\\( a, b, c ) -> Json.Encode.list identity [ {} a, {} b, {} c ]",
            T::elm_type(),
            U::elm_type(),
            V::elm_type(),
        )
    }

    fn encoder_definition() -> Option<String> {
        None
    }
}

impl<'a, T: Elm + ToOwned + ?Sized> Elm for std::borrow::Cow<'a, T> {
    fn elm_type() -> String {
        format!("{}", T::elm_type())
    }

    fn elm_definition() -> Option<String> {
        T::elm_definition()
    }

    fn decoder_type() -> String {
        format!("{}", T::decoder_type())
    }

    fn decoder_definition() -> Option<String> {
        T::decoder_definition()
    }

    fn encoder_type() -> String {
        format!("{}", T::encoder_type())
    }

    fn encoder_definition() -> Option<String> {
        T::encoder_definition()
    }
}

impl<T, const U: usize> Elm for [T; U]
where
    T: Elm,
{
    fn elm_type() -> String {
        <[T]>::elm_type()
    }

    fn elm_definition() -> Option<String> {
        <[T]>::elm_definition()
    }

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

macro_rules! impl_builtin {
    ($rust_type: ty, $elm_type: expr, $elm_decoder: expr, $elm_encoder: expr) => {
        impl Elm for $rust_type {
            fn elm_type() -> String {
                $elm_type.to_string()
            }

            fn elm_definition() -> Option<String> {
                None
            }

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
        impl<T: Elm> Elm for $rust_type {
            fn elm_type() -> String {
                format!("{} ({})", $elm_name, T::elm_type())
            }

            fn elm_definition() -> Option<String> {
                None
            }

            fn decoder_type() -> String {
                format!("{} ({})", $elm_decoder, T::decoder_type())
            }

            fn decoder_definition() -> Option<String> {
                None
            }

            fn encoder_type() -> String {
                format!("{} ({})", $elm_encoder, T::encoder_type())
            }

            fn encoder_definition() -> Option<String> {
                None
            }
        }
    };
}

macro_rules! impl_builtin_map {
    ($rust_type: ty) => {
        impl<T: Elm> Elm for $rust_type {
            fn elm_type() -> String {
                format!("Dict String ({})", T::elm_type())
            }

            fn elm_definition() -> Option<String> {
                None
            }

            fn decoder_type() -> String {
                format!("Json.Decode.dict ({})", T::decoder_type())
            }

            fn decoder_definition() -> Option<String> {
                None
            }

            fn encoder_type() -> String {
                format!("Json.Encode.dict identity ({})", T::encoder_type())
            }

            fn encoder_definition() -> Option<String> {
                None
            }
        }
    };
}

macro_rules! impl_builtin_ptr {
    ($rust_type: ty) => {
        impl<T: Elm + ?Sized> Elm for $rust_type {
            fn elm_type() -> String {
                format!("{}", T::elm_type())
            }

            fn elm_definition() -> Option<String> {
                T::elm_definition()
            }

            fn decoder_type() -> String {
                format!("{}", T::decoder_type())
            }

            fn decoder_definition() -> Option<String> {
                T::decoder_definition()
            }

            fn encoder_type() -> String {
                format!("{}", T::encoder_type())
            }

            fn encoder_definition() -> Option<String> {
                T::encoder_definition()
            }
        }
    };
}

macro_rules! impl_builtin_ptr_borrow {
    ($rust_type: ty) => {
        impl<'a, T: Elm + ?Sized> Elm for $rust_type {
            fn elm_type() -> String {
                format!("{}", T::elm_type())
            }

            fn elm_definition() -> Option<String> {
                T::elm_definition()
            }

            fn decoder_type() -> String {
                format!("{}", T::decoder_type())
            }

            fn decoder_definition() -> Option<String> {
                T::decoder_definition()
            }

            fn encoder_type() -> String {
                format!("{}", T::encoder_type())
            }

            fn encoder_definition() -> Option<String> {
                T::encoder_definition()
            }
        }
    };
}

impl_builtin_ptr_borrow!(&'a T);
impl_builtin_ptr_borrow!(&'a mut T);
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
// todo duration
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
// todo phantomdata?
impl_builtin_ptr!(std::rc::Rc<T>);
impl_builtin_ptr!(std::cell::RefCell<T>);
// todo result
impl_builtin_ptr!(std::sync::RwLock<T>);
// todo socketaddrs
impl_builtin!(String, "String", "Json.Decode.string", "Json.Encode.string");
// todo systemtime
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
impl<T: chrono::TimeZone> Elm for chrono::DateTime<T> {
    fn elm_type() -> String {
        String::elm_type()
    }

    fn elm_definition() -> Option<String> {
        String::elm_definition()
    }

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
