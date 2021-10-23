mod elm;
mod json;
mod rocket;

use std::borrow::Cow;

use heck::{CamelCase, KebabCase, MixedCase, ShoutyKebabCase, ShoutySnakeCase, SnakeCase};
use proc_macro::TokenStream;
use syn::{
    Attribute, Data, DataEnum, DeriveInput, Fields, Generics, Ident, Lit, Meta, MetaList,
    NestedMeta, Type,
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
    ident: Ident,
    generics: Generics,
    type_info: TypeInfo,
}

#[derive(Default)]
struct ContainerAttributes {
    serde_rename_all: Option<RenameAll>,
    serde_enum_representation: Option<EnumRepresentation>,
    serde_transparent: bool,
}

#[derive(Clone, Copy)]
enum RenameAll {
    Lowercase,
    Uppercase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

enum Rename {
    Container(RenameAll),
    Field(String),
}

enum EnumRepresentation {
    Tag(String),
    TagContent(String, String),
    Untagged,
}

#[derive(Default)]
struct FieldAttributes {
    serde_rename: Option<String>,
}

enum TypeInfo {
    // struct S;
    // null
    Unit,
    // struct S(String);
    // "string"
    Newtype(Box<Type>),
    // struct S(String, u32);
    // []
    // ["string", 0]
    Tuple(Vec<Type>),
    // struct S {
    //     s: String,
    // }
    // {}
    // {"s": "string"}
    Struct(Vec<StructField>),
    // enum E {
    //     Variant,
    // }
    Enum(Vec<EnumVariant>),
}

struct StructField {
    ident: Ident,
    rename: Option<Rename>,
    ty: Type,
}

impl StructField {
    fn name<'a>(&'a self) -> Cow<'a, str> {
        match &self.rename {
            Some(Rename::Container(rename_all)) => match rename_all {
                RenameAll::Lowercase => self.ident.to_string().to_lowercase().into(),
                RenameAll::Uppercase => self.ident.to_string().to_uppercase().into(),
                RenameAll::PascalCase => self.ident.to_string().to_mixed_case().into(),
                RenameAll::CamelCase => self.ident.to_string().to_camel_case().into(),
                RenameAll::SnakeCase => self.ident.to_string().to_snake_case().into(),
                RenameAll::ScreamingSnakeCase => {
                    self.ident.to_string().to_shouty_snake_case().into()
                }
                RenameAll::KebabCase => self.ident.to_string().to_kebab_case().into(),
                RenameAll::ScreamingKebabCase => {
                    self.ident.to_string().to_shouty_kebab_case().into()
                }
            },
            Some(Rename::Field(rename)) => rename.into(),
            None => self.ident.to_string().into(),
        }
    }
}

struct EnumVariant {
    ident: Ident,
    variant: EnumKind,
}

enum EnumKind {
    // Variant,
    // "Variant"
    Unit,
    // Variant(String),
    // {"Variant": "string"}
    Newtype(Box<Type>),
    // Variant(String, u32),
    // {"Variant": []}
    // {"Variant": ["string", 0]}
    Tuple(Vec<Type>),
    // Variant {
    //     s: String,
    // }
    // {}
    // {"s": "string"}
    Struct(Vec<StructField>),
}

// parses the input to an intermediate representation that's convenient to turn into the end result
fn derive_input_to_intermediate(input: DeriveInput) -> Intermediate {
    let container_attributes = parse_container_attributes(&input.attrs);
    let type_info = parse_type_info(input.data, &container_attributes);
    Intermediate {
        ident: input.ident,
        generics: input.generics,
        type_info,
    }
}

fn parse_type_info(data: Data, attributes: &ContainerAttributes) -> TypeInfo {
    match data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Unit => TypeInfo::Unit,
            Fields::Unnamed(mut unnamed) => {
                if unnamed.unnamed.len() == 1 {
                    TypeInfo::Newtype(Box::new(unnamed.unnamed.pop().unwrap().into_value().ty))
                } else {
                    TypeInfo::Tuple(unnamed.unnamed.into_iter().map(|field| field.ty).collect())
                }
            }
            Fields::Named(mut named) => {
                if attributes.serde_transparent && named.named.len() == 1 {
                    TypeInfo::Newtype(Box::new(named.named.pop().unwrap().into_value().ty))
                } else {
                    TypeInfo::Struct(
                        named
                            .named
                            .into_iter()
                            .map(|field| {
                                let ident = field.ident.unwrap();
                                let field_attributes = parse_field_attributes(&field.attrs);
                                StructField {
                                    ident,
                                    rename: field_attributes
                                        .serde_rename
                                        .map(Rename::Field)
                                        .or(attributes.serde_rename_all.map(Rename::Container)),
                                    ty: field.ty,
                                }
                            })
                            .collect(),
                    )
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            if variants.is_empty() {
                panic!("empty enums are not supported");
            }
            TypeInfo::Enum(
                variants
                    .into_iter()
                    .map(|variant| {
                        parse_variant_attributes(&variant.attrs);
                        match variant.fields {
                            Fields::Unit => EnumVariant {
                                ident: variant.ident,
                                variant: EnumKind::Unit,
                            },
                            Fields::Unnamed(mut unnamed) => {
                                if unnamed.unnamed.len() == 1 {
                                    EnumVariant {
                                        ident: variant.ident,
                                        variant: EnumKind::Newtype(Box::new(
                                            unnamed.unnamed.pop().unwrap().into_value().ty,
                                        )),
                                    }
                                } else {
                                    EnumVariant {
                                        ident: variant.ident,
                                        variant: EnumKind::Tuple(
                                            unnamed
                                                .unnamed
                                                .into_iter()
                                                .map(|field| field.ty)
                                                .collect(),
                                        ),
                                    }
                                }
                            }
                            Fields::Named(named) => EnumVariant {
                                ident: variant.ident,
                                variant: EnumKind::Struct(
                                    named
                                        .named
                                        .into_iter()
                                        .map(|field| StructField {
                                            ident: field.ident.unwrap(),
                                            rename: attributes
                                                .serde_rename_all
                                                .map(Rename::Container),
                                            ty: field.ty,
                                        })
                                        .collect(),
                                ),
                            },
                        }
                    })
                    .collect(),
            )
        }
        Data::Union(_) => panic!("unions are not supported"),
    }
}

fn parse_container_attributes(attrs: &[Attribute]) -> ContainerAttributes {
    let mut attributes = ContainerAttributes::default();
    for attr in attrs {
        if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
            if meta_list.path.is_ident("jalava") {
                //
            } else if meta_list.path.is_ident("serde") {
                parse_serde_container_attributes(&mut attributes, meta_list);
            } else if meta_list.path.is_ident("rocket") {
                //
            }
        }
    }
    attributes
}

fn parse_serde_container_attributes(attributes: &mut ContainerAttributes, meta_list: MetaList) {
    let mut nested = meta_list.nested.into_iter();
    match nested.next() {
        Some(NestedMeta::Meta(Meta::NameValue(name_value))) => {
            if name_value.path.is_ident("rename_all") {
                if let Lit::Str(rename_all) = name_value.lit {
                    match rename_all.value().as_str() {
                        "lowercase" => attributes.serde_rename_all = Some(RenameAll::Lowercase),
                        "UPPERCASE" => attributes.serde_rename_all = Some(RenameAll::Uppercase),
                        "PascalCase" => attributes.serde_rename_all = Some(RenameAll::PascalCase),
                        "camelCase" => attributes.serde_rename_all = Some(RenameAll::CamelCase),
                        "snake_case" => attributes.serde_rename_all = Some(RenameAll::SnakeCase),
                        "SCREAMING_SNAKE_CASE" => {
                            attributes.serde_rename_all = Some(RenameAll::ScreamingSnakeCase);
                        }
                        "kebab-case" => attributes.serde_rename_all = Some(RenameAll::KebabCase),
                        "SCREAMING-KEBAB-CASE" => {
                            attributes.serde_rename_all = Some(RenameAll::ScreamingKebabCase);
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
                                );
                            }
                        }
                    } else {
                        attributes.serde_enum_representation =
                            Some(EnumRepresentation::Tag(tag.value()));
                    }
                }
            } else if name_value.path.is_ident("content") {
                if let Lit::Str(content) = name_value.lit {
                    if let Some(NestedMeta::Meta(Meta::NameValue(inner_name_value))) = nested.next()
                    {
                        if inner_name_value.path.is_ident("tag") {
                            if let Lit::Str(tag) = inner_name_value.lit {
                                attributes.serde_enum_representation = Some(
                                    EnumRepresentation::TagContent(content.value(), tag.value()),
                                );
                            }
                        }
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
}

fn parse_variant_attributes(_attrs: &[Attribute]) -> () {
    // todo
}

fn parse_field_attributes(attrs: &[Attribute]) -> FieldAttributes {
    let mut attributes = FieldAttributes::default();

    for attr in attrs {
        if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
            if meta_list.path.is_ident("jalava") {
                //
            } else if meta_list.path.is_ident("serde") {
                parse_serde_field_attributes(&mut attributes, meta_list);
            } else if meta_list.path.is_ident("rocket") {
                //
            }
        }
    }

    attributes
}

fn parse_serde_field_attributes(attributes: &mut FieldAttributes, meta_list: MetaList) {
    let mut nested = meta_list.nested.into_iter();
    match nested.next() {
        Some(NestedMeta::Meta(Meta::NameValue(name_value))) => {
            if name_value.path.is_ident("rename") {
                if let Lit::Str(rename) = name_value.lit {
                    attributes.serde_rename = Some(rename.value())
                }
            }
        }
        Some(NestedMeta::Lit(Lit::Str(s))) => match s.value().as_str() {
            _ => {}
        },
        _ => {}
    }
}
