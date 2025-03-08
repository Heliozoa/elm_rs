# elm_rs_derive

[![Crates.io](https://img.shields.io/crates/v/elm_rs_derive)](https://crates.io/crates/elm_rs_derive)
[![docs.rs](https://img.shields.io/badge/docs.rs-elm_rs_derive-success)](https://docs.rs/elm_rs_derive)
[![Crates.io](https://img.shields.io/crates/l/elm_rs_derive)](https://choosealicense.com/licenses/mpl-2.0/)
[![GitHub](https://img.shields.io/badge/GitHub-Heliozoa-24292f)](https://github.com/Heliozoa/elm_rs_derive)

Derive macros for `elm_rs::{Elm, ElmEncode, ElmDecode, ElmQuery, ElmQueryField}`.

## Features
- `default`: None.
- `json`: Enables the derive macros for `ElmEncode` and `ElmDecode`.
- `query`: Enables the derive macro for `ElmQuery` and `ElmQueryField`.
- `serde`: Enables compatibility with serde attributes like `#[serde(rename_all = "camelCase")]`.
