//! Derive macros for elm_rs.

mod attributes;
mod elm;
#[cfg(feature = "json")]
mod json;
#[cfg(feature = "query")]
mod query;

use self::attributes::*;
use heck::{ToLowerCamelCase, ToPascalCase};
use proc_macro::TokenStream;
use proc_macro2::Span;
use std::borrow::Cow;
use syn::{
    punctuated::Punctuated, spanned::Spanned, Data, DataEnum, DeriveInput, Fields, FieldsNamed,
    Generics, Ident, Type, Variant,
};

/// Derive `Elm`.
#[proc_macro_derive(Elm)]
pub fn derive_elm(input: TokenStream) -> TokenStream {
    elm::derive(input)
}

/// Derive `ElmJson`.
#[cfg(feature = "json")]
#[proc_macro_derive(ElmJson)]
pub fn derive_elm_json(input: TokenStream) -> TokenStream {
    json::derive(input)
}

/// Derive `ElmQuery`.
#[cfg(feature = "query")]
#[proc_macro_derive(ElmQuery)]
pub fn derive_elm_query(input: TokenStream) -> TokenStream {
    query::derive(input)
}

/// Intermediate representation of the derive input for more convenient handling.
struct Intermediate {
    ident: Ident,
    elm_type: String,
    generics: Generics,
    generics_without_bounds: Generics,
    type_info: TypeInfo,
}

enum TypeInfo {
    // struct S;
    Unit,
    // struct S(String);
    Newtype(Box<Type>),
    // struct S(String, u32);
    Tuple(Vec<Type>),
    // struct S {
    //     s: String,
    // }
    Struct(Vec<StructField>),
    // enum E {
    //     Variant,
    // }
    Enum {
        representation: EnumRepresentation,
        variants: Vec<EnumVariant>,
    },
}

struct StructField {
    ident: Ident,
    rename: Option<Rename>,
    rename_deserialize: Option<Rename>,
    rename_serialize: Option<Rename>,
    // todo
    // aliases: Vec<String>,
    ty: Type,
}

impl StructField {
    /// The name in the Elm type definition. Always camelCased for consistency with Elm style guidelines.
    fn name_elm(&self) -> String {
        self.ident.to_string().to_lower_camel_case()
    }

    /// The name when deserializing from JSON in Elm.
    /// The name when serializing to JSON in Rust.
    fn name_deserialize(&'_ self) -> Cow<'_, str> {
        match &self.rename_deserialize {
            Some(Rename::Container(rename_all)) => rename_all.rename_ident(&self.ident).into(),
            Some(Rename::Field(rename)) => rename.into(),
            None => match &self.rename {
                Some(Rename::Container(rename_all)) => rename_all.rename_ident(&self.ident).into(),
                Some(Rename::Field(rename)) => rename.into(),
                None => self.ident.to_string().into(),
            },
        }
    }

    /// The name when serializing to JSON in Elm.
    /// The name when deserializing from JSON in Rust.
    fn name_serialize(&'_ self) -> Cow<'_, str> {
        match &self.rename_serialize {
            Some(Rename::Container(rename_all)) => rename_all.rename_ident(&self.ident).into(),
            Some(Rename::Field(rename)) => rename.into(),
            None => match &self.rename {
                Some(Rename::Container(rename_all)) => rename_all.rename_ident(&self.ident).into(),
                Some(Rename::Field(rename)) => rename.into(),
                None => self.ident.to_string().into(),
            },
        }
    }
}

enum Rename {
    Container(RenameAll),
    Field(String),
}

struct EnumVariant {
    ident: Ident,
    rename: Option<Rename>,
    // corresponds to serde's rename serialize
    rename_deserialize: Option<Rename>,
    // corresponds to serde's rename deserialize
    rename_serialize: Option<Rename>,
    skip: bool,
    other: bool,
    variant: EnumVariantKind,
    span: Span,
}

pub(crate) enum EnumRepresentation {
    External,
    Internal { tag: String },
    Adjacent { tag: String, content: String },
    Untagged,
}

impl Default for EnumRepresentation {
    fn default() -> Self {
        Self::External
    }
}

impl EnumVariant {
    /// The name in the Elm type definition. Always PascalCased for consistency with Elm style guidelines.
    fn name_elm(&'_ self) -> Cow<'_, str> {
        self.ident.to_string().to_pascal_case().into()
    }

    /// The name when deserializing from JSON in Elm.
    /// The name when serializing to JSON in Rust.
    fn name_deserialize(&'_ self) -> Cow<'_, str> {
        match &self.rename_deserialize {
            Some(Rename::Container(rename_all)) => rename_all.rename_ident(&self.ident).into(),
            Some(Rename::Field(rename)) => rename.into(),
            None => match &self.rename {
                Some(Rename::Container(rename_all)) => rename_all.rename_ident(&self.ident).into(),
                Some(Rename::Field(rename)) => rename.into(),
                None => self.ident.to_string().into(),
            },
        }
    }

    /// The name when serializing to JSON in Elm.
    /// The name when deserializing from JSON in Rust.
    fn name_serialize(&'_ self) -> Cow<'_, str> {
        match &self.rename_serialize {
            Some(Rename::Container(rename_all)) => rename_all.rename_ident(&self.ident).into(),
            Some(Rename::Field(rename)) => rename.into(),
            None => match &self.rename {
                Some(Rename::Container(rename_all)) => rename_all.rename_ident(&self.ident).into(),
                Some(Rename::Field(rename)) => rename.into(),
                None => self.ident.to_string().into(),
            },
        }
    }
}

enum EnumVariantKind {
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
fn derive_input_to_intermediate(input: DeriveInput) -> Result<Intermediate, syn::Error> {
    let container_attributes = parse_container_attributes(&input.attrs);
    let type_info = parse_type_info(input.data, container_attributes)?;

    let elm_type = input.ident.to_string().to_pascal_case();
    let mut generics_without_bounds = input.generics.clone();
    for p in generics_without_bounds.type_params_mut() {
        p.bounds = Punctuated::default();
    }
    Ok(Intermediate {
        ident: input.ident,
        elm_type,
        generics: input.generics,
        generics_without_bounds,
        type_info,
    })
}

fn parse_type_info(
    data: Data,
    container_attributes: ContainerAttributes,
) -> Result<TypeInfo, syn::Error> {
    let type_info = match data {
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
                if container_attributes.serde_transparent && named.named.len() == 1 {
                    TypeInfo::Newtype(Box::new(named.named.pop().unwrap().into_value().ty))
                } else {
                    TypeInfo::Struct(fields_named_to_struct_fields(named, &container_attributes))
                }
            }
        },
        Data::Enum(DataEnum { variants, .. }) => {
            if variants.is_empty() {
                return Err(syn::Error::new(
                    variants.span(),
                    "empty enums are not supported",
                ));
            }
            let variants = variants
                .into_iter()
                .map(|variant| parse_enum_variant(variant, &container_attributes))
                .collect::<Result<_, _>>()?;

            TypeInfo::Enum {
                representation: container_attributes.serde_enum_representation,
                variants,
            }
        }
        Data::Union(union) => {
            return Err(syn::Error::new(
                union.union_token.span(),
                "unions are not supported",
            ))
        }
    };
    Ok(type_info)
}

fn parse_enum_variant(
    variant: Variant,
    container_attributes: &ContainerAttributes,
) -> Result<EnumVariant, syn::Error> {
    let span = variant.span();
    let variant_attributes = parse_variant_attributes(&variant.attrs);
    let variant_kind = match variant.fields {
        Fields::Unit => EnumVariantKind::Unit,
        Fields::Unnamed(mut unnamed) if unnamed.unnamed.len() == 1 => {
            EnumVariantKind::Newtype(Box::new(unnamed.unnamed.pop().unwrap().into_value().ty))
        }
        Fields::Unnamed(unnamed) => {
            EnumVariantKind::Tuple(unnamed.unnamed.into_iter().map(|field| field.ty).collect())
        }
        Fields::Named(named) => {
            EnumVariantKind::Struct(fields_named_to_struct_fields(named, container_attributes))
        }
    };
    let variant = EnumVariant {
        ident: variant.ident,
        rename: variant_attributes
            .serde_rename
            .map(Rename::Field)
            .or_else(|| variant_attributes.serde_rename_all.map(Rename::Container))
            .or_else(|| container_attributes.serde_rename_all.map(Rename::Container)),
        rename_deserialize: variant_attributes
            .serde_rename_serialize
            .map(Rename::Field)
            .or_else(|| {
                variant_attributes
                    .serde_rename_all_serialize
                    .map(Rename::Container)
            })
            .or_else(|| {
                container_attributes
                    .serde_rename_all_serialize
                    .map(Rename::Container)
            }),
        rename_serialize: variant_attributes
            .serde_rename_deserialize
            .map(Rename::Field)
            .or_else(|| {
                variant_attributes
                    .serde_rename_all_deserialize
                    .map(Rename::Container)
            })
            .or_else(|| {
                container_attributes
                    .serde_rename_all_deserialize
                    .map(Rename::Container)
            }),
        skip: variant_attributes.serde_skip,
        other: variant_attributes.serde_other,
        variant: variant_kind,
        span,
    };
    Ok(variant)
}

fn fields_named_to_struct_fields(
    named: FieldsNamed,
    container_attributes: &ContainerAttributes,
) -> Vec<StructField> {
    named
        .named
        .into_iter()
        .map(|field| {
            let field_attributes = parse_field_attributes(&field.attrs);
            (field, field_attributes)
        })
        .filter(|(_, field_attributes)| !field_attributes.serde_skip)
        .map(|(field, field_attributes)| StructField {
            ident: field.ident.unwrap(),
            rename: field_attributes
                .serde_rename
                .map(Rename::Field)
                .or_else(|| container_attributes.serde_rename_all.map(Rename::Container)),
            rename_deserialize: field_attributes
                .serde_rename_serialize
                .map(Rename::Field)
                .or_else(|| {
                    container_attributes
                        .serde_rename_all_serialize
                        .map(Rename::Container)
                }),
            rename_serialize: field_attributes
                .serde_rename_deserialize
                .map(Rename::Field)
                .or_else(|| {
                    container_attributes
                        .serde_rename_all_deserialize
                        .map(Rename::Container)
                }),
            // todo
            // aliases: field_attributes.serde_aliases,
            ty: field.ty,
        })
        .collect()
}
