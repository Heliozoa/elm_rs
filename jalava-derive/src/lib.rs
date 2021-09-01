mod elm;
mod json;
mod rocket;

use proc_macro::TokenStream;
use syn::{Data, DataEnum, DeriveInput, Fields, Generics, Ident, Type};

/// Derive `Elm` for any type with fields that all implement `Elm`
/// The big exception are tuples larger than 3, since 3-tuples are the largest that exist in Elm.
#[proc_macro_derive(Elm)]
pub fn derive_elm(input: TokenStream) -> TokenStream {
    elm::derive_elm(input)
}

/// Derive `ElmJson` for any type other than empty enums or unions.
#[proc_macro_derive(ElmJson)]
pub fn derive_elm_json(input: TokenStream) -> TokenStream {
    json::derive_elm_json(input)
}

/// Derive `ElmForm` for any struct.
#[proc_macro_derive(ElmForm)]
pub fn derive_elm_form(input: TokenStream) -> TokenStream {
    rocket::derive_elm_form(input)
}

/// Derive `ElmFormParts` for any type other than empty enums, enums with variants that have fields, or unions.
#[proc_macro_derive(ElmFormParts)]
pub fn derive_elm_form_parts(input: TokenStream) -> TokenStream {
    rocket::derive_elm_form_parts(input)
}

struct Intermediate {
    ident: Ident,
    generics: Generics,
    kind: TypeKind,
}

enum TypeKind {
    // struct S;
    // null
    Unit,
    // struct S(String);
    // "string"
    Newtype(Type),
    // struct S(String, u32);
    // []
    // ["string", 0]
    Tuple(Vec<Type>),
    // struct S {
    //     s: String,
    // }
    // {}
    // {"s": "string"}
    Struct(Vec<(Ident, Type)>),
    // enum E {
    //     Variant,
    // }
    Enum(Vec<(Ident, EnumVariant)>),
}

enum EnumVariant {
    // Variant,
    // "Variant"
    Unit,
    // Variant(String),
    // {"Variant": "string"}
    Newtype(Type),
    // Variant(String, u32),
    // {"Variant": []}
    // {"Variant": ["string", 0]}
    Tuple(Vec<Type>),
    // Variant {
    //     s: String,
    // }
    // {}
    // {"s": "string"}
    Struct(Vec<(Ident, Type)>),
}

// parses the input to an intermediate representation that's convenient to turn into the end result
fn derive_input_to_intermediate(input: DeriveInput) -> Intermediate {
    if input.generics.lt_token.is_some() {
        // panic!("{:?}", generics)
    }

    let type_kind = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Unit => TypeKind::Unit,
            Fields::Unnamed(mut unnamed) => {
                if unnamed.unnamed.len() == 1 {
                    TypeKind::Newtype(unnamed.unnamed.pop().unwrap().into_value().ty)
                } else {
                    TypeKind::Tuple(unnamed.unnamed.into_iter().map(|u| u.ty).collect())
                }
            }
            Fields::Named(named) => TypeKind::Struct(
                named
                    .named
                    .into_iter()
                    .map(|f| (f.ident.unwrap(), f.ty))
                    .collect(),
            ),
        },
        Data::Enum(DataEnum { variants, .. }) => {
            if variants.is_empty() {
                panic!("empty enums not supported");
            }
            TypeKind::Enum(
                variants
                    .into_iter()
                    .map(|v| {
                        (
                            v.ident,
                            match v.fields {
                                Fields::Unit => EnumVariant::Unit,
                                Fields::Unnamed(mut unnamed) => {
                                    if unnamed.unnamed.len() == 1 {
                                        EnumVariant::Newtype(
                                            unnamed.unnamed.pop().unwrap().into_value().ty,
                                        )
                                    } else {
                                        EnumVariant::Tuple(
                                            unnamed.unnamed.into_iter().map(|u| u.ty).collect(),
                                        )
                                    }
                                }
                                Fields::Named(named) => EnumVariant::Struct(
                                    named
                                        .named
                                        .into_iter()
                                        .map(|f| (f.ident.unwrap(), f.ty))
                                        .collect(),
                                ),
                            },
                        )
                    })
                    .collect(),
            )
        }
        Data::Union(_) => panic!("unions are not supported"),
    };
    Intermediate {
        ident: input.ident,
        generics: input.generics,
        kind: type_kind,
    }
}
