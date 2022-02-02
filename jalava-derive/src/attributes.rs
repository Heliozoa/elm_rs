//! Parsing macro attributes.

use crate::EnumRepresentation;
use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnakeCase, ToSnakeCase,
};
use proc_macro2::Ident;
use syn::{Attribute, Meta};

#[derive(Default)]
pub(crate) struct ContainerAttributes {
    pub serde_rename_all: Option<RenameAll>,
    pub serde_rename_all_deserialize: Option<RenameAll>,
    pub serde_rename_all_serialize: Option<RenameAll>,
    pub serde_enum_representation: EnumRepresentation,
    pub serde_transparent: bool,
}

#[derive(Clone, Copy)]
pub enum RenameAll {
    Lowercase,
    Uppercase,
    PascalCase,
    CamelCase,
    SnakeCase,
    ScreamingSnakeCase,
    KebabCase,
    ScreamingKebabCase,
}

impl RenameAll {
    pub fn from(s: &str) -> Option<Self> {
        match s {
            "lowercase" => Some(RenameAll::Lowercase),
            "UPPERCASE" => Some(RenameAll::Uppercase),
            "PascalCase" => Some(RenameAll::PascalCase),
            "camelCase" => Some(RenameAll::CamelCase),
            "snake_case" => Some(RenameAll::SnakeCase),
            "SCREAMING_SNAKE_CASE" => Some(RenameAll::ScreamingSnakeCase),
            "kebab-case" => Some(RenameAll::KebabCase),
            "SCREAMING-KEBAB-CASE" => Some(RenameAll::ScreamingKebabCase),
            _ => None,
        }
    }

    pub fn rename_ident(self, ident: &Ident) -> String {
        match self {
            RenameAll::Lowercase => ident.to_string().to_lowercase(),
            RenameAll::Uppercase => ident.to_string().to_uppercase(),
            RenameAll::PascalCase => ident.to_string().to_pascal_case(),
            RenameAll::CamelCase => ident.to_string().to_lower_camel_case(),
            RenameAll::SnakeCase => ident.to_string().to_snake_case(),
            RenameAll::ScreamingSnakeCase => ident.to_string().to_shouty_snake_case(),
            RenameAll::KebabCase => ident.to_string().to_kebab_case(),
            RenameAll::ScreamingKebabCase => ident.to_string().to_shouty_kebab_case(),
        }
    }
}

#[derive(Default)]
pub struct VariantAttributes {
    pub serde_rename: Option<String>,
    pub serde_rename_deserialize: Option<String>,
    pub serde_rename_serialize: Option<String>,
    pub serde_rename_all: Option<RenameAll>,
    pub serde_rename_all_deserialize: Option<RenameAll>,
    pub serde_rename_all_serialize: Option<RenameAll>,
    pub serde_aliases: Vec<String>,
    pub serde_skip: bool,
    pub serde_other: bool,
}

#[derive(Default)]
pub struct FieldAttributes {
    pub serde_rename: Option<String>,
    pub serde_rename_deserialize: Option<String>,
    pub serde_rename_serialize: Option<String>,
    pub serde_aliases: Vec<String>,
    pub serde_flatten: bool,
    pub serde_skip: bool,
}

pub(crate) fn parse_container_attributes(attrs: &[Attribute]) -> ContainerAttributes {
    let mut attributes = ContainerAttributes::default();
    for attr in attrs {
        if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
            if meta_list.path.is_ident("jalava") {
                // todo
            } else if meta_list.path.is_ident("serde") {
                #[cfg(feature = "serde")]
                serde::parse_container_attributes(&mut attributes, meta_list);
            }
        }
    }
    attributes
}

pub(crate) fn parse_variant_attributes(attrs: &[Attribute]) -> VariantAttributes {
    let mut attributes = VariantAttributes::default();

    for attr in attrs {
        if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
            if meta_list.path.is_ident("jalava") {
                // todo
            } else if meta_list.path.is_ident("serde") {
                #[cfg(feature = "serde")]
                serde::parse_variant_attributes(&mut attributes, meta_list);
            }
        }
    }

    attributes
}

pub(crate) fn parse_field_attributes(attrs: &[Attribute]) -> FieldAttributes {
    let mut attributes = FieldAttributes::default();
    for attr in attrs {
        if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
            if meta_list.path.is_ident("jalava") {
                // todo
            } else if meta_list.path.is_ident("serde") {
                #[cfg(feature = "serde")]
                serde::parse_field_attributes(&mut attributes, meta_list);
            }
        }
    }
    attributes
}

#[cfg(feature = "serde")]
mod serde {
    use crate::EnumRepresentation;

    use super::*;
    use syn::{Lit, Meta, MetaList, MetaNameValue, NestedMeta, Path};

    pub(super) fn parse_container_attributes(
        attributes: &mut ContainerAttributes,
        meta_list: MetaList,
    ) {
        let mut nested = meta_list.nested.into_iter().peekable();
        match nested.next() {
            Some(NestedMeta::Meta(Meta::List(list))) => {
                if list.path.is_ident("rename_all") {
                    for nested in list.nested {
                        if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                            path,
                            lit: Lit::Str(rename),
                            ..
                        })) = nested
                        {
                            let rename = RenameAll::from(&rename.value());
                            if path.is_ident("deserialize") {
                                attributes.serde_rename_all_deserialize = rename;
                            } else if path.is_ident("serialize") {
                                attributes.serde_rename_all_serialize = rename;
                            }
                        }
                    }
                }
            }
            Some(NestedMeta::Meta(Meta::NameValue(name_value))) => {
                if name_value.path.is_ident("rename_all") {
                    if let Lit::Str(rename_all) = name_value.lit {
                        attributes.serde_rename_all = RenameAll::from(&rename_all.value())
                    }
                } else if name_value.path.is_ident("tag") {
                    if let Lit::Str(tag) = name_value.lit {
                        if let Some(NestedMeta::Meta(Meta::NameValue(inner_name_value))) =
                            nested.next()
                        {
                            if inner_name_value.path.is_ident("content") {
                                if let Lit::Str(content) = inner_name_value.lit {
                                    attributes.serde_enum_representation =
                                        EnumRepresentation::Adjacent {
                                            tag: tag.value(),
                                            content: content.value(),
                                        };
                                }
                            }
                        } else {
                            attributes.serde_enum_representation =
                                EnumRepresentation::Internal { tag: tag.value() };
                        }
                    }
                } else if name_value.path.is_ident("content") {
                    if let Lit::Str(content) = name_value.lit {
                        if let Some(NestedMeta::Meta(Meta::NameValue(inner_name_value))) =
                            nested.next()
                        {
                            if inner_name_value.path.is_ident("tag") {
                                if let Lit::Str(tag) = inner_name_value.lit {
                                    attributes.serde_enum_representation =
                                        EnumRepresentation::Adjacent {
                                            tag: tag.value(),
                                            content: content.value(),
                                        };
                                }
                            }
                        }
                    }
                }
            }
            Some(NestedMeta::Meta(Meta::Path(Path { segments, .. }))) => {
                for segment in segments {
                    match segment.ident.to_string().as_str() {
                        "transparent" => attributes.serde_transparent = true,
                        "untagged" => {
                            attributes.serde_enum_representation = EnumRepresentation::Untagged
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) fn parse_variant_attributes(
        attributes: &mut VariantAttributes,
        meta_list: MetaList,
    ) {
        let mut nested = meta_list.nested.into_iter();
        match nested.next() {
            Some(NestedMeta::Meta(Meta::List(list))) => {
                if list.path.is_ident("rename") {
                    for nested in list.nested {
                        if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                            path,
                            lit: Lit::Str(rename),
                            ..
                        })) = nested
                        {
                            if path.is_ident("deserialize") {
                                attributes.serde_rename_deserialize = Some(rename.value());
                            } else if path.is_ident("serialize") {
                                attributes.serde_rename_serialize = Some(rename.value());
                            }
                        }
                    }
                } else if list.path.is_ident("rename_all") {
                    for nested in list.nested {
                        if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                            path,
                            lit: Lit::Str(rename),
                            ..
                        })) = nested
                        {
                            let rename = RenameAll::from(&rename.value());
                            if path.is_ident("deserialize") {
                                attributes.serde_rename_all_deserialize = rename;
                            } else if path.is_ident("serialize") {
                                attributes.serde_rename_all_serialize = rename;
                            }
                        }
                    }
                }
            }
            Some(NestedMeta::Meta(Meta::NameValue(name_value))) => {
                if name_value.path.is_ident("rename") {
                    if let Lit::Str(rename) = name_value.lit {
                        attributes.serde_rename = Some(rename.value())
                    }
                } else if name_value.path.is_ident("rename_all") {
                    if let Lit::Str(rename_all) = name_value.lit {
                        attributes.serde_rename_all = RenameAll::from(&rename_all.value())
                    }
                } else if name_value.path.is_ident("alias") {
                    if let Lit::Str(rename_all) = name_value.lit {
                        attributes.serde_aliases.push(rename_all.value());
                    }
                }
            }
            Some(NestedMeta::Meta(Meta::Path(Path { segments, .. }))) => {
                for segment in segments {
                    match segment.ident.to_string().as_str() {
                        "other" => attributes.serde_other = true,
                        "skip" => attributes.serde_skip = true,
                        _ => {}
                    }
                }
            }
            _ => {}
        }
    }

    pub(super) fn parse_field_attributes(attributes: &mut FieldAttributes, meta_list: MetaList) {
        let mut nested = meta_list.nested.into_iter();
        match nested.next() {
            Some(NestedMeta::Meta(Meta::List(list))) => {
                if list.path.is_ident("rename") {
                    for nested in list.nested {
                        if let NestedMeta::Meta(Meta::NameValue(MetaNameValue {
                            path,
                            lit: Lit::Str(rename),
                            ..
                        })) = nested
                        {
                            if path.is_ident("deserialize") {
                                attributes.serde_rename_deserialize = Some(rename.value());
                            } else if path.is_ident("serialize") {
                                attributes.serde_rename_serialize = Some(rename.value());
                            }
                        }
                    }
                }
            }
            Some(NestedMeta::Meta(Meta::NameValue(name_value))) => {
                if name_value.path.is_ident("rename") {
                    if let Lit::Str(rename) = name_value.lit {
                        attributes.serde_rename = Some(rename.value())
                    }
                } else if name_value.path.is_ident("alias") {
                    if let Lit::Str(rename_all) = name_value.lit {
                        attributes.serde_aliases.push(rename_all.value());
                    }
                }
            }
            Some(NestedMeta::Meta(Meta::Path(path))) => {
                if path.is_ident("flatten") {
                    attributes.serde_flatten = true
                } else if path.is_ident("skip") {
                    attributes.serde_skip = true
                }
            }
            _ => {}
        }
    }
}
