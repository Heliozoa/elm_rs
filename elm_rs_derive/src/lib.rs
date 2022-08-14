//! Derive macros for elm_rs.

mod attributes;
mod elm;
#[cfg(feature = "json")]
mod elm_decode;
#[cfg(feature = "json")]
mod elm_encode;
#[cfg(feature = "query")]
mod elm_query;
#[cfg(feature = "query")]
mod elm_query_field;

use self::attributes::{ContainerAttributes, FieldAttributes, VariantAttributes};
use heck::{ToLowerCamelCase, ToPascalCase};
use proc_macro::TokenStream;
use proc_macro2::{Span, TokenStream as TokenStream2};
use quote::ToTokens;
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

/// Derive `ElmEncode`.
#[cfg(feature = "json")]
#[proc_macro_derive(ElmEncode)]
pub fn derive_elm_serialize(input: TokenStream) -> TokenStream {
    elm_encode::derive(input)
}

/// Derive `ElmDecode`.
#[cfg(feature = "json")]
#[proc_macro_derive(ElmDecode)]
pub fn derive_elm_deserialize(input: TokenStream) -> TokenStream {
    elm_decode::derive(input)
}

/// Derive `ElmQuery`.
#[cfg(feature = "query")]
#[proc_macro_derive(ElmQuery)]
pub fn derive_elm_query(input: TokenStream) -> TokenStream {
    elm_query::derive(input)
}

/// Derive `ElmQueryField`.
#[cfg(feature = "query")]
#[proc_macro_derive(ElmQueryField)]
pub fn derive_elm_query_field(input: TokenStream) -> TokenStream {
    elm_query_field::derive(input)
}

/// Intermediate representation of the derive input for more convenient handling.
struct Intermediate {
    ident: Ident,
    elm_type: String,
    generics: Generics,
    generics_without_bounds: Generics,
    type_info: TypeInfo,
    container_attributes: ContainerAttributes,
}

impl Intermediate {
    // parses the input to an intermediate representation that's convenient to turn into the end result
    fn parse(input: DeriveInput) -> Result<Self, syn::Error> {
        let container_attributes = ContainerAttributes::parse(&input.attrs);
        let type_info = TypeInfo::parse(input.data, &container_attributes)?;

        let elm_type = input.ident.to_string().to_pascal_case();
        let mut generics_without_bounds = input.generics.clone();
        for p in generics_without_bounds.type_params_mut() {
            p.bounds = Punctuated::default();
        }
        Ok(Self {
            ident: input.ident,
            elm_type,
            generics: input.generics,
            generics_without_bounds,
            type_info,
            container_attributes,
        })
    }
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
        variants: Vec<EnumVariant>,
        #[cfg(feature = "serde")]
        representation: attributes::serde::EnumRepresentation,
    },
}

impl TypeInfo {
    pub fn parse(
        data: Data,
        container_attributes: &ContainerAttributes,
    ) -> Result<Self, syn::Error> {
        let type_info = match data {
            Data::Struct(data_struct) => match data_struct.fields {
                Fields::Unit => TypeInfo::Unit,
                Fields::Unnamed(unnamed) => {
                    if unnamed.unnamed.len() == 1 {
                        TypeInfo::Newtype(Box::new(unnamed.unnamed.into_iter().next().unwrap().ty))
                    } else {
                        TypeInfo::Tuple(unnamed.unnamed.into_iter().map(|field| field.ty).collect())
                    }
                }
                Fields::Named(named) => {
                    #[cfg(not(feature = "serde"))]
                    let transparent = false;
                    #[cfg(feature = "serde")]
                    let transparent = container_attributes.serde.transparent;
                    if transparent && named.named.len() == 1 {
                        TypeInfo::Newtype(Box::new(named.named.into_iter().next().unwrap().ty))
                    } else {
                        TypeInfo::Struct(StructField::parse(named))
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
                    .map(EnumVariant::parse)
                    .collect::<Result<_, _>>()?;

                TypeInfo::Enum {
                    #[cfg(feature = "serde")]
                    representation: container_attributes.serde.enum_representation.clone(),
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
}

struct StructField {
    ident: Ident,
    // todo
    // aliases: Vec<String>,
    ty: TokenStream2,
    #[cfg(feature = "serde")]
    serde_attributes: attributes::serde::FieldAttributes,
}

impl StructField {
    /// The name in the Elm type definition. Always camelCased for consistency with Elm style guidelines.
    fn name_elm(&self) -> String {
        self.ident.to_string().to_lower_camel_case()
    }

    #[cfg(feature = "json")]
    fn name_encode(&self, container_attributes: &ContainerAttributes) -> String {
        // rename during Rust serialization = needs rename during Elm deserialization
        // explicit rename has priority
        #[cfg(feature = "serde")]
        if let Some(rename) = self
            .serde_attributes
            .rename
            .as_ref()
            .or(self.serde_attributes.rename_deserialize.as_ref())
        {
            rename.to_string()
        } else if let Some(rename_all) = container_attributes
            .serde
            .rename_all
            .or(container_attributes.serde.rename_all_deserialize)
        {
            rename_all.rename_ident(&self.ident)
        } else {
            self.ident.to_string()
        }
        #[cfg(not(feature = "serde"))]
        self.ident.to_string()
    }

    #[cfg(feature = "json")]
    fn name_decode(&self, container_attributes: &ContainerAttributes) -> String {
        // rename during Rust deserialization = needs rename during Elm serialization
        // explicit rename has priority
        #[cfg(feature = "serde")]
        if let Some(rename) = self
            .serde_attributes
            .rename
            .as_ref()
            .or(self.serde_attributes.rename_serialize.as_ref())
        {
            rename.to_string()
        } else if let Some(rename_all) = container_attributes
            .serde
            .rename_all
            .or(container_attributes.serde.rename_all_serialize)
        {
            rename_all.rename_ident(&self.ident)
        } else {
            self.ident.to_string()
        }
        #[cfg(not(feature = "serde"))]
        self.ident.to_string()
    }

    fn parse(fields: FieldsNamed) -> Vec<Self> {
        let fields = fields.named.into_iter().map(|field| {
            let field_attributes = FieldAttributes::parse(&field.attrs);
            (field, field_attributes)
        });
        #[cfg(feature = "serde")]
        let fields = fields.filter(|(_, field_attributes)| !field_attributes.serde.skip);
        fields
            .map(|(field, field_attributes)| {
                StructField {
                    ident: field.ident.unwrap(), // only tuple struct fields are unnamed
                    // todo
                    // aliases: field_attributes.serde.aliases,
                    ty: field.ty.to_token_stream(),
                    #[cfg(feature = "serde")]
                    serde_attributes: field_attributes.serde,
                }
            })
            .collect()
    }
}

struct EnumVariant {
    ident: Ident,
    variant: EnumVariantKind,
    span: Span,
    #[cfg(feature = "serde")]
    serde_attributes: attributes::serde::VariantAttributes,
}

impl EnumVariant {
    /// The name in the Elm type definition. Always PascalCased for consistency with Elm style guidelines.
    fn name_elm(&'_ self) -> Cow<'_, str> {
        self.ident.to_string().to_pascal_case().into()
    }

    #[cfg(feature = "json")]
    fn name_encode(&self, container_attributes: &ContainerAttributes) -> String {
        // rename during Rust deserialization = needs rename during Elm encoding
        // explicit rename has priority
        #[cfg(feature = "serde")]
        if let Some(rename) = self
            .serde_attributes
            .rename
            .as_ref()
            .or(self.serde_attributes.rename_deserialize.as_ref())
        {
            rename.clone()
        } else if let Some(rename_all) = container_attributes
            .serde
            .rename_all
            .or(container_attributes.serde.rename_all_deserialize)
        {
            rename_all.rename_ident(&self.ident)
        } else {
            self.ident.to_string()
        }
        #[cfg(not(feature = "serde"))]
        self.ident.to_string()
    }

    #[cfg(feature = "json")]
    fn name_decode(&self, container_attributes: &ContainerAttributes) -> String {
        // rename during Rust serialization = needs rename during Elm decoding
        // explicit rename has priority
        #[cfg(feature = "serde")]
        if let Some(rename) = self
            .serde_attributes
            .rename
            .as_ref()
            .or(self.serde_attributes.rename_serialize.as_ref())
        {
            rename.to_string()
        } else if let Some(rename_all) = container_attributes
            .serde
            .rename_all
            .or(container_attributes.serde.rename_all_serialize)
        {
            rename_all.rename_ident(&self.ident)
        } else {
            self.ident.to_string()
        }
        #[cfg(not(feature = "serde"))]
        self.ident.to_string()
    }

    fn parse(variant: Variant) -> Result<Self, syn::Error> {
        let span = variant.span();
        let variant_attributes = VariantAttributes::parse(&variant.attrs);
        let variant_kind = match variant.fields {
            Fields::Unit => EnumVariantKind::Unit,
            Fields::Unnamed(unnamed) if unnamed.unnamed.len() == 1 => EnumVariantKind::Newtype(
                unnamed
                    .unnamed
                    .into_iter()
                    .next()
                    .unwrap()
                    .ty
                    .to_token_stream(),
            ),
            Fields::Unnamed(unnamed) => EnumVariantKind::Tuple(
                unnamed
                    .unnamed
                    .into_iter()
                    .map(|field| field.ty.to_token_stream())
                    .collect(),
            ),
            Fields::Named(named) => EnumVariantKind::Struct(StructField::parse(named)),
        };
        let variant = EnumVariant {
            ident: variant.ident,
            variant: variant_kind,
            span,
            #[cfg(feature = "serde")]
            serde_attributes: variant_attributes.serde,
        };
        Ok(variant)
    }
}

enum EnumVariantKind {
    // Variant,
    // "Variant"
    Unit,
    // Variant(String),
    // {"Variant": "string"}
    Newtype(TokenStream2), // e.g. Vec<i32>
    // Variant(String, u32),
    // {"Variant": []}
    // {"Variant": ["string", 0]}
    Tuple(Vec<TokenStream2>), // e.g. [Vec<i32>, String]
    // Variant {
    //     s: String,
    // }
    // {}
    // {"s": "string"}
    Struct(Vec<StructField>),
}
