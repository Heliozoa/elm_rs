//! Contains the rocket-compatible `ElmForm` trait.

pub use jalava_derive::{ElmForm, ElmFormParts};
use rocket::fs::TempFile;

/// Used to create multipart requests to a Rocket server.
pub trait ElmForm {
    /// A function that creates a Http.Body from the corresponding Elm type.
    fn prepare_form() -> String;
}

/// Used to for types that can be fields in a multipart request to a Rocket server.
pub trait ElmFormParts {
    fn form_parts(field: &str) -> String {
        Self::form_parts_inner(field, &format!("form.{}", field), 0)
    }

    fn form_parts_inner(field: &str, path: &str, recursion: u32) -> String;

    fn to_string() -> Option<String> {
        None
    }

    fn to_string_definition() -> Option<String> {
        None
    }
}

impl ElmFormParts for TempFile<'_> {
    fn form_parts_inner(field: &str, path: &str, _recursion: u32) -> String {
        format!(r#"[ Http.filePart ("{}") {} ]"#, field, path)
    }
}

impl<T: ElmFormParts> ElmFormParts for Vec<T> {
    fn form_parts_inner(field: &str, path: &str, recursion: u32) -> String {
        let idx = format!("i{}", recursion);
        let var = format!("x{}", recursion);
        format!(
            r#"List.concat (List.concat (List.indexedMap (\{} {} -> [{}]) ({} {})))"#,
            idx,
            var,
            T::form_parts_inner(
                &format!("{}[\" ++ String.fromInt {} ++ \"]", field, idx),
                &var,
                recursion + 1
            ),
            T::to_string().unwrap_or("identity".to_string()),
            path
        )
    }
}

macro_rules! impl_stringpart {
    ($ty: ty, $to_string: expr) => {
        impl ElmFormParts for $ty {
            fn form_parts_inner(field: &str, path: &str, _recursion: u32) -> String {
                format!(
                    r#"[ Http.stringPart "{}" ({} {}) ]"#,
                    field, $to_string, path
                )
            }
        }
    };
}

impl_stringpart!(String, "identity");
impl_stringpart!(&'_ str, "identity");
impl_stringpart!(bool, "boolToString");
impl_stringpart!(u8, "String.fromInt");
impl_stringpart!(u16, "String.fromInt");
impl_stringpart!(u32, "String.fromInt");
impl_stringpart!(u64, "String.fromInt");
impl_stringpart!(u128, "String.fromInt");
impl_stringpart!(usize, "String.fromInt");
impl_stringpart!(i8, "String.fromInt");
impl_stringpart!(i16, "String.fromInt");
impl_stringpart!(i32, "String.fromInt");
impl_stringpart!(i64, "String.fromInt");
impl_stringpart!(i128, "String.fromInt");
impl_stringpart!(isize, "String.fromInt");
impl_stringpart!(f32, "String.fromFloat");
impl_stringpart!(f64, "String.fromFloat");
impl_stringpart!(std::num::NonZeroU8, "String.fromInt");
impl_stringpart!(std::num::NonZeroU16, "String.fromInt");
impl_stringpart!(std::num::NonZeroU32, "String.fromInt");
impl_stringpart!(std::num::NonZeroU64, "String.fromInt");
impl_stringpart!(std::num::NonZeroU128, "String.fromInt");
impl_stringpart!(std::num::NonZeroUsize, "String.fromInt");
impl_stringpart!(std::num::NonZeroI8, "String.fromInt");
impl_stringpart!(std::num::NonZeroI16, "String.fromInt");
impl_stringpart!(std::num::NonZeroI32, "String.fromInt");
impl_stringpart!(std::num::NonZeroI64, "String.fromInt");
impl_stringpart!(std::num::NonZeroI128, "String.fromInt");
impl_stringpart!(std::num::NonZeroIsize, "String.fromInt");
// todo ipaddrs, socketaddrs
impl_stringpart!(std::borrow::Cow<'_, str>, "identity");
#[cfg(feature = "time")]
impl_stringpart!(time::Date, "identity");
#[cfg(feature = "time")]
impl_stringpart!(time::Time, "identity");
#[cfg(feature = "time")]
impl_stringpart!(time::PrimitiveDateTime, "identity");

macro_rules! impl_dict {
    ($ty:ty) => {
        impl<T: ElmFormParts> ElmFormParts for std::collections::HashMap<$ty, T> {
            fn form_parts_inner(field: &str, path: &str, recursion: u32) -> String {
                let key = format!("k{}", recursion);
                let val = format!("v{}", recursion);
                format!(
                    r#"List.concat (List.map (\( {}, {} ) -> ({})) (Dict.toList {}))"#,
                    key,
                    val,
                    T::form_parts_inner(
                        &format!("{}[\" ++ {} ++ \"]", field, key),
                        &val,
                        recursion + 1
                    ),
                    path
                )
            }
        }
    };
}

impl_dict!(String);
impl_dict!(&'_ str);
impl_dict!(u8);
impl_dict!(u16);
impl_dict!(u32);
impl_dict!(u64);
impl_dict!(u128);
impl_dict!(usize);
impl_dict!(i8);
impl_dict!(i16);
impl_dict!(i32);
impl_dict!(i64);
impl_dict!(i128);
impl_dict!(isize);
impl_dict!(f32);
impl_dict!(f64);
