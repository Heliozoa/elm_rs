//! Contains the `ElmQuery` trait.

#[cfg(feature = "jalava-derive")]
pub use jalava_derive::ElmQuery;

/// Used to generate URL encoded key-value pairs in Elm.
pub trait ElmQuery {
    /// A function that takes the
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

pub trait ElmQueryField {
    fn field_type() -> &'static str;
}

impl<T: ElmQueryField + ?Sized> ElmQueryField for &'_ T {
    fn field_type() -> &'static str {
        T::field_type()
    }
}

impl<T: ElmQueryField + ?Sized> ElmQueryField for &'_ mut T {
    fn field_type() -> &'static str {
        T::field_type()
    }
}

macro_rules! impl_for {
    ($e:expr, $($t:ty),+) => {
        $(
            impl ElmQueryField for $t {
                fn field_type() -> &'static str {
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
    fn field_type() -> &'static str {
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
