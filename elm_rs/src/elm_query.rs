//! Contains the `ElmQuery` trait.

#[cfg(feature = "derive")]
pub use elm_rs_derive::ElmQuery;
#[cfg(feature = "derive")]
pub use elm_rs_derive::ElmQueryField;

/// Used to generate URL encoded key-value pairs in Elm.
pub trait ElmQuery {
    /// Generates an Elm function that creates a `List Url.Builder.QueryParameter`.
    fn elm_query() -> String;
}

impl<T: ElmQuery + ?Sized> ElmQuery for &'_ T {
    fn elm_query() -> String {
        T::elm_query()
    }
}

impl<T: ElmQuery + ?Sized> ElmQuery for &'_ mut T {
    fn elm_query() -> String {
        T::elm_query()
    }
}

/// Used to generate the fields for `ElmQuery::elm_query`.
pub trait ElmQueryField {
    /// The `Url.Builder` type of the field (either `Url.Builder.string` or `Url.Builder.int`).
    fn query_field_type() -> &'static str;
    /// The name of the Elm function used to transform it a String or Int for the `query_field_type` (usually `identity`).
    fn query_field_encoder_name() -> &'static str {
        "identity"
    }
    /// If the type needs a custom encoder, this function generates its definition.
    fn query_field_encoder_definition() -> Option<String> {
        None
    }
}

impl<T: ElmQueryField + ?Sized> ElmQueryField for &'_ T {
    fn query_field_type() -> &'static str {
        T::query_field_type()
    }
}

impl<T: ElmQueryField + ?Sized> ElmQueryField for &'_ mut T {
    fn query_field_type() -> &'static str {
        T::query_field_type()
    }
}

macro_rules! impl_for {
    ($e:expr, $($t:ty),+) => {
        $(
            impl ElmQueryField for $t {
                fn query_field_type() -> &'static str {
                    $e
                }
            }
        )*
    };
}

impl_for!(
    "Url.Builder.string",
    String,
    str,
    std::path::Path,
    std::path::PathBuf
);
#[cfg(feature = "uuid")]
impl_for!("Url.Builder.string", uuid::Uuid);
#[cfg(feature = "chrono")]
impl_for!(
    "Url.Builder.string",
    chrono::NaiveTime,
    chrono::NaiveDate,
    chrono::NaiveDateTime
);
#[cfg(feature = "chrono")]
impl<T: chrono::TimeZone> ElmQueryField for chrono::DateTime<T> {
    fn query_field_type() -> &'static str {
        "Url.Builder.string"
    }
}

impl_for!(
    "Url.Builder.int",
    u8,
    u16,
    u32,
    u64,
    u128,
    usize,
    i8,
    i16,
    i32,
    i64,
    i128,
    isize,
    std::sync::atomic::AtomicU8,
    std::sync::atomic::AtomicU16,
    std::sync::atomic::AtomicU32,
    std::sync::atomic::AtomicU64,
    std::sync::atomic::AtomicUsize,
    std::sync::atomic::AtomicI8,
    std::sync::atomic::AtomicI16,
    std::sync::atomic::AtomicI32,
    std::sync::atomic::AtomicI64,
    std::sync::atomic::AtomicIsize,
    std::num::NonZeroU8,
    std::num::NonZeroU16,
    std::num::NonZeroU32,
    std::num::NonZeroU64,
    std::num::NonZeroU128,
    std::num::NonZeroUsize,
    std::num::NonZeroI8,
    std::num::NonZeroI16,
    std::num::NonZeroI32,
    std::num::NonZeroI64,
    std::num::NonZeroI128,
    std::num::NonZeroIsize
);
