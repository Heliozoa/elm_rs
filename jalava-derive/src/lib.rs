mod elm;
mod json;
mod rocket;

use heck::{CamelCase, KebabCase, MixedCase, ShoutyKebabCase, ShoutySnakeCase, SnakeCase};
use proc_macro::TokenStream;
use syn::{
    Data, DataEnum, DeriveInput, Fields, Generics, Ident, Lit, Meta, MetaList, NestedMeta, Type,
};

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
    attributes: Attributes,
    ident: Ident,
    generics: Generics,
    kind: TypeKind,
}

#[derive(Default)]
struct Attributes {
    serde_rename_all: Option<Rename>,
    serde_enum_representation: Option<EnumRepresentation>,
    serde_transparent: bool,
}

impl Attributes {
    fn merge(self, merge: Self) -> Self {
        Attributes {
            serde_rename_all: self.serde_rename_all.or(merge.serde_rename_all),
            serde_enum_representation: self
                .serde_enum_representation
                .or(merge.serde_enum_representation),
            serde_transparent: self.serde_transparent || merge.serde_transparent,
        }
    }
}

enum Rename {
    Lowercase,
    Uppercase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

enum EnumRepresentation {
    Tag(String),
    TagContent(String, String),
    Untagged,
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
    let type_kind = parse_type_kind(input.data);

    let mut attributes = Attributes::default();
    for attr in input.attrs {
        if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
            if meta_list.path.is_ident("jalava") {
                //
            } else if meta_list.path.is_ident("serde") {
                attributes = attributes.merge(parse_serde_attributes(meta_list))
            } else if meta_list.path.is_ident("rocket") {
                //
            }
        }
    }
    Intermediate {
        attributes,
        ident: input.ident,
        generics: input.generics,
        kind: type_kind,
    }
}

fn parse_type_kind(data: Data) -> TypeKind {
    match data {
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
                panic!("empty enums are not supported");
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
    }
}

fn parse_serde_attributes(meta_list: MetaList) -> Attributes {
    let mut attributes = Attributes::default();

    let mut nested = meta_list.nested.into_iter();
    match nested.next() {
        Some(NestedMeta::Meta(Meta::NameValue(name_value))) => {
            if name_value.path.is_ident("rename_all") {
                if let Lit::Str(s) = name_value.lit {
                    match s.value().as_str() {
                        "lowercase" => attributes.serde_rename_all = Some(Rename::Lowercase),
                        "UPPERCASE" => attributes.serde_rename_all = Some(Rename::Uppercase),
                        "PascalCase" => attributes.serde_rename_all = Some(Rename::PascalCase),
                        "camelCase" => attributes.serde_rename_all = Some(Rename::CamelCase),
                        "snake_case" => attributes.serde_rename_all = Some(Rename::SnakeCase),
                        "SCREAMING_SNAKE_CASE" => {
                            attributes.serde_rename_all = Some(Rename::ScreamingSnakeCase)
                        }
                        "kebab-case" => attributes.serde_rename_all = Some(Rename::KebabCase),
                        "SCREAMING-KEBAB-CASE" => {
                            attributes.serde_rename_all = Some(Rename::ScreamingKebabCase)
                        }
                        _ => {}
                    }
                }
            } else if name_value.path.is_ident("tag") {
                if let Lit::Str(tag) = name_value.lit {
                    if let Some(NestedMeta::Meta(Meta::NameValue(inner_name_value))) = nested.next()
                    {
                        if inner_name_value.path.is_ident("content") {
                            if let Lit::Str(content) = inner_name_value.lit {
                                attributes.serde_enum_representation = Some(
                                    EnumRepresentation::TagContent(tag.value(), content.value()),
                                )
                            }
                        }
                    } else {
                        attributes.serde_enum_representation =
                            Some(EnumRepresentation::Tag(tag.value()))
                    }
                }
            }
        }
        Some(NestedMeta::Lit(Lit::Str(s))) => match s.value().as_str() {
            "untagged" => attributes.serde_enum_representation = Some(EnumRepresentation::Untagged),
            "transparent" => attributes.serde_transparent = true,
            _ => {}
        },
        _ => {}
    }
    attributes
}

fn convert_case(i: Ident, attributes: &Attributes) -> String {
    let i = i.to_string();
    match attributes.serde_rename_all {
        Some(Rename::Lowercase) => i.to_lowercase(),
        Some(Rename::Uppercase) => i.to_uppercase(),
        Some(Rename::PascalCase) => i.to_mixed_case(),
        Some(Rename::CamelCase) => i.to_camel_case(),
        Some(Rename::SnakeCase) => i.to_snake_case(),
        Some(Rename::ScreamingSnakeCase) => i.to_shouty_snake_case(),
        Some(Rename::KebabCase) => i.to_kebab_case(),
        Some(Rename::ScreamingKebabCase) => i.to_shouty_kebab_case(),
        None => i,
    }
}
