//! Parsing macro attributes.

use syn::{Attribute, Meta};

#[derive(Default)]
pub struct ContainerAttributes {
    #[cfg(feature = "serde")]
    pub serde: serde::ContainerAttributes,
}

impl ContainerAttributes {
    pub fn parse(attrs: &[Attribute]) -> Self {
        let mut attributes = Self::default();
        for attr in attrs {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                /* todo
                if meta_list.path.is_ident("elm_rs") {
                }
                */
                #[cfg(feature = "serde")]
                if meta_list.path.is_ident("serde") {
                    attributes.serde.parse(meta_list);
                }
            }
        }
        attributes
    }
}

#[derive(Default)]
pub struct VariantAttributes {
    #[cfg(feature = "serde")]
    pub serde: serde::VariantAttributes,
}

impl VariantAttributes {
    pub fn parse(attrs: &[Attribute]) -> Self {
        let mut attributes = Self::default();

        for attr in attrs {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                /* todo
                if meta_list.path.is_ident("elm_rs") {
                }
                */
                #[cfg(feature = "serde")]
                if meta_list.path.is_ident("serde") {
                    attributes.serde.parse(meta_list);
                }
            }
        }

        attributes
    }
}

#[derive(Default)]
pub struct FieldAttributes {
    #[cfg(feature = "serde")]
    pub serde: serde::FieldAttributes,
}

impl FieldAttributes {
    pub fn parse(attrs: &[Attribute]) -> Self {
        let mut attributes = Self::default();
        for attr in attrs {
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                /* todo
                if meta_list.path.is_ident("elm_rs") {
                }
                */
                #[cfg(feature = "serde")]
                if meta_list.path.is_ident("serde") {
                    attributes.serde.parse(meta_list);
                }
            }
        }
        attributes
    }
}

#[cfg(feature = "serde")]
pub mod serde {
    use heck::{
        ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnakeCase,
        ToSnakeCase,
    };
    use proc_macro2::Ident;
    use syn::{Lit, Meta, MetaList, MetaNameValue, NestedMeta, Path};

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
    pub struct ContainerAttributes {
        pub rename_all: Option<RenameAll>,
        pub rename_all_deserialize: Option<RenameAll>,
        pub rename_all_serialize: Option<RenameAll>,
        pub enum_representation: EnumRepresentation,
        pub transparent: bool,
    }

    impl ContainerAttributes {
        pub fn parse(&mut self, meta_list: MetaList) {
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
                                    self.rename_all_deserialize = rename;
                                } else if path.is_ident("serialize") {
                                    self.rename_all_serialize = rename;
                                }
                            }
                        }
                    }
                }
                Some(NestedMeta::Meta(Meta::NameValue(name_value))) => {
                    if name_value.path.is_ident("rename_all") {
                        if let Lit::Str(rename_all) = name_value.lit {
                            self.rename_all = RenameAll::from(&rename_all.value())
                        }
                    } else if name_value.path.is_ident("tag") {
                        if let Lit::Str(tag) = name_value.lit {
                            if let Some(NestedMeta::Meta(Meta::NameValue(inner_name_value))) =
                                nested.next()
                            {
                                if inner_name_value.path.is_ident("content") {
                                    if let Lit::Str(content) = inner_name_value.lit {
                                        self.enum_representation = EnumRepresentation::Adjacent {
                                            tag: tag.value(),
                                            content: content.value(),
                                        };
                                    }
                                }
                            } else {
                                self.enum_representation =
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
                                        self.enum_representation = EnumRepresentation::Adjacent {
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
                            "transparent" => self.transparent = true,
                            "untagged" => self.enum_representation = EnumRepresentation::Untagged,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    #[derive(Default)]
    pub struct VariantAttributes {
        pub rename: Option<String>,
        pub rename_deserialize: Option<String>,
        pub rename_serialize: Option<String>,
        pub rename_all: Option<RenameAll>,
        pub rename_all_deserialize: Option<RenameAll>,
        pub rename_all_serialize: Option<RenameAll>,
        pub aliases: Vec<String>,
        pub skip: bool,
        pub other: bool,
    }

    impl VariantAttributes {
        pub fn parse(&mut self, meta_list: MetaList) {
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
                                    self.rename_deserialize = Some(rename.value());
                                } else if path.is_ident("serialize") {
                                    self.rename_serialize = Some(rename.value());
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
                                    self.rename_all_deserialize = rename;
                                } else if path.is_ident("serialize") {
                                    self.rename_all_serialize = rename;
                                }
                            }
                        }
                    }
                }
                Some(NestedMeta::Meta(Meta::NameValue(name_value))) => {
                    if name_value.path.is_ident("rename") {
                        if let Lit::Str(rename) = name_value.lit {
                            self.rename = Some(rename.value())
                        }
                    } else if name_value.path.is_ident("rename_all") {
                        if let Lit::Str(rename_all) = name_value.lit {
                            self.rename_all = RenameAll::from(&rename_all.value())
                        }
                    } else if name_value.path.is_ident("alias") {
                        if let Lit::Str(rename_all) = name_value.lit {
                            self.aliases.push(rename_all.value());
                        }
                    }
                }
                Some(NestedMeta::Meta(Meta::Path(Path { segments, .. }))) => {
                    for segment in segments {
                        match segment.ident.to_string().as_str() {
                            "other" => self.other = true,
                            "skip" => self.skip = true,
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }

    #[derive(Default)]
    pub struct FieldAttributes {
        pub rename: Option<String>,
        pub rename_deserialize: Option<String>,
        pub rename_serialize: Option<String>,
        pub aliases: Vec<String>,
        pub flatten: bool,
        pub skip: bool,
    }

    impl FieldAttributes {
        pub fn parse(&mut self, meta_list: MetaList) {
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
                                    self.rename_deserialize = Some(rename.value());
                                } else if path.is_ident("serialize") {
                                    self.rename_serialize = Some(rename.value());
                                }
                            }
                        }
                    }
                }
                Some(NestedMeta::Meta(Meta::NameValue(name_value))) => {
                    if name_value.path.is_ident("rename") {
                        if let Lit::Str(rename) = name_value.lit {
                            self.rename = Some(rename.value())
                        }
                    } else if name_value.path.is_ident("alias") {
                        if let Lit::Str(rename_all) = name_value.lit {
                            self.aliases.push(rename_all.value());
                        }
                    }
                }
                Some(NestedMeta::Meta(Meta::Path(path))) => {
                    if path.is_ident("flatten") {
                        self.flatten = true
                    } else if path.is_ident("skip") {
                        self.skip = true
                    }
                }
                _ => {}
            }
        }
    }

    #[derive(Default, Clone)]
    pub enum EnumRepresentation {
        #[default]
        External,
        Internal {
            tag: String,
        },
        Adjacent {
            tag: String,
            content: String,
        },
        Untagged,
    }
}
