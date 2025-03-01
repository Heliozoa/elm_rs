//! Parsing macro attributes.

use syn::Attribute;

#[derive(Default)]
pub struct ContainerAttributes {
    #[cfg(feature = "serde")]
    pub serde: serde::ContainerAttributes,
}

impl ContainerAttributes {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut attributes = Self::default();
        for attr in attrs {
            #[cfg(feature = "serde")]
            if attr.path().is_ident("serde") {
                attributes.serde.parse(attr)?;
            }
        }
        Ok(attributes)
    }
}

#[derive(Default)]
pub struct VariantAttributes {
    #[cfg(feature = "serde")]
    pub serde: serde::VariantAttributes,
}

impl VariantAttributes {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut attributes = Self::default();

        for attr in attrs {
            #[cfg(feature = "serde")]
            if attr.path().is_ident("serde") {
                attributes.serde.parse(attr)?;
            }
        }

        Ok(attributes)
    }
}

#[derive(Default)]
pub struct FieldAttributes {
    #[cfg(feature = "serde")]
    pub serde: serde::FieldAttributes,
}

impl FieldAttributes {
    pub fn parse(attrs: &[Attribute]) -> syn::Result<Self> {
        let mut attributes = Self::default();

        for attr in attrs {
            #[cfg(feature = "serde")]
            if attr.path().is_ident("serde") {
                attributes.serde.parse(attr)?;
            }
        }

        Ok(attributes)
    }
}

#[cfg(feature = "serde")]
pub mod serde {
    use heck::{
        ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnakeCase,
        ToSnakeCase,
    };
    use proc_macro2::Ident;
    use syn::{token, Attribute, LitStr, Token};

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
        pub rename: Option<String>,
        pub rename_deserialize: Option<String>,
        pub rename_serialize: Option<String>,
        pub rename_all: Option<RenameAll>,
        pub rename_all_deserialize: Option<RenameAll>,
        pub rename_all_serialize: Option<RenameAll>,
        pub rename_all_fields: Option<RenameAll>,
        pub rename_all_fields_deserialize: Option<RenameAll>,
        pub rename_all_fields_serialize: Option<RenameAll>,
        pub enum_representation: EnumRepresentation,
        pub transparent: bool,
    }

    impl ContainerAttributes {
        pub fn parse(&mut self, attr: &Attribute) -> syn::Result<()> {
            let mut tag_attr = None;
            let mut content_attr = None;

            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    // rename(..) or rename = ".."
                    if meta.input.peek(token::Paren) {
                        // rename(..)
                        meta.parse_nested_meta(|meta| {
                            if meta.input.parse::<Token![=]>().is_ok() {
                                if meta.path.is_ident("deserialize") {
                                    let content = meta.input.parse::<LitStr>()?;
                                    self.rename_deserialize = Some(content.value());
                                }

                                if meta.path.is_ident("serialize") {
                                    let content = meta.input.parse::<LitStr>()?;
                                    self.rename_serialize = Some(content.value());
                                }
                            }

                            Ok(())
                        })?;
                    } else if meta.input.parse::<Token![=]>().is_ok() {
                        // rename = ".."
                        let content = meta.input.parse::<LitStr>()?;
                        self.rename = Some(content.value());
                    }
                }

                if meta.path.is_ident("rename_all") {
                    // rename_all(..) or rename_all = ".."
                    if meta.input.peek(token::Paren) {
                        // rename_all(..)
                        meta.parse_nested_meta(|meta| {
                            if meta.input.parse::<Token![=]>().is_ok() {
                                if meta.path.is_ident("deserialize") {
                                    let content = meta.input.parse::<LitStr>()?;
                                    self.rename_all_deserialize = RenameAll::from(&content.value());
                                }

                                if meta.path.is_ident("serialize") {
                                    let content = meta.input.parse::<LitStr>()?;
                                    self.rename_all_serialize = RenameAll::from(&content.value());
                                }
                            }

                            Ok(())
                        })?;
                    } else if meta.input.parse::<Token![=]>().is_ok() {
                        // rename_all = ".."
                        let content = meta.input.parse::<LitStr>()?;
                        self.rename_all = RenameAll::from(&content.value());
                    }
                }

                if meta.path.is_ident("rename_all_fields") {
                    // rename_all_fields(..) or rename_all_fields = ".."
                    if meta.input.peek(token::Paren) {
                        // rename_all_fields(..)
                        meta.parse_nested_meta(|meta| {
                            if meta.input.parse::<Token![=]>().is_ok() {
                                if meta.path.is_ident("deserialize") {
                                    let content = meta.input.parse::<LitStr>()?;
                                    self.rename_all_fields_deserialize =
                                        RenameAll::from(&content.value());
                                }

                                if meta.path.is_ident("serialize") {
                                    let content = meta.input.parse::<LitStr>()?;
                                    self.rename_all_fields_serialize =
                                        RenameAll::from(&content.value());
                                }
                            }

                            Ok(())
                        })?;
                    } else if meta.input.parse::<Token![=]>().is_ok() {
                        // rename_all_fields = ".."
                        let content = meta.input.parse::<LitStr>()?;
                        self.rename_all_fields = RenameAll::from(&content.value());
                    }
                }

                if meta.path.is_ident("tag") {
                    // tag = ".."
                    if meta.input.parse::<Token![=]>().is_ok() {
                        let content = meta.input.parse::<LitStr>()?;
                        tag_attr = Some(content.value());
                    }
                }

                if meta.path.is_ident("content") {
                    // content = ".."
                    if meta.input.parse::<Token![=]>().is_ok() {
                        let content = meta.input.parse::<LitStr>()?;
                        content_attr = Some(content.value());
                    }
                }

                if meta.path.is_ident("untagged") {
                    // untagged
                    self.enum_representation = EnumRepresentation::Untagged;
                }

                if meta.path.is_ident("transparent") {
                    self.transparent = true;
                }

                // we don't need to handle all serde attributes
                Ok(())
            })?;

            if let Some(tag) = tag_attr {
                if let Some(content) = content_attr {
                    self.enum_representation = EnumRepresentation::Adjacent { tag, content };
                } else {
                    self.enum_representation = EnumRepresentation::Internal { tag };
                }
            }

            Ok(())
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
        pub fn parse(&mut self, attr: &Attribute) -> syn::Result<()> {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    // rename(..) or rename = ".."
                    if meta.input.peek(token::Paren) {
                        // rename(..)
                        meta.parse_nested_meta(|meta| {
                            if meta.input.parse::<Token![=]>().is_ok() {
                                if meta.path.is_ident("deserialize") {
                                    let content = meta.input.parse::<LitStr>()?;
                                    self.rename_deserialize = Some(content.value());
                                }

                                if meta.path.is_ident("serialize") {
                                    let content = meta.input.parse::<LitStr>()?;
                                    self.rename_serialize = Some(content.value());
                                }
                            }

                            Ok(())
                        })?;
                    } else if meta.input.parse::<Token![=]>().is_ok() {
                        // rename = ".."
                        let content = meta.input.parse::<LitStr>()?;
                        self.rename = Some(content.value());
                    }
                }

                if meta.path.is_ident("alias") && meta.input.parse::<Token![=]>().is_ok() {
                    // alias = ".."
                    let content = meta.input.parse::<LitStr>()?;
                    self.aliases.push(content.value());
                }

                if meta.path.is_ident("rename_all") {
                    // rename_all(..) or rename_all = ".."
                    if meta.input.peek(token::Paren) {
                        // rename_all(..)
                        meta.parse_nested_meta(|meta| {
                            if meta.input.parse::<Token![=]>().is_ok() {
                                if meta.path.is_ident("deserialize") {
                                    let content = meta.input.parse::<LitStr>()?;
                                    self.rename_all_deserialize = RenameAll::from(&content.value());
                                }

                                if meta.path.is_ident("serialize") {
                                    let content = meta.input.parse::<LitStr>()?;
                                    self.rename_all_serialize = RenameAll::from(&content.value());
                                }
                            }

                            Ok(())
                        })?;
                    } else if meta.input.parse::<Token![=]>().is_ok() {
                        // rename_all = ".."
                        let content = meta.input.parse::<LitStr>()?;
                        self.rename_all = RenameAll::from(&content.value());
                    }
                }

                if meta.path.is_ident("skip") {
                    self.skip = true;
                }

                if meta.path.is_ident("other") {
                    self.other = true;
                }

                Ok(())
            })?;

            Ok(())
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
        pub fn parse(&mut self, attr: &Attribute) -> syn::Result<()> {
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename") {
                    // rename(..) or rename = ".."
                    if meta.input.peek(token::Paren) {
                        // rename(..)
                        meta.parse_nested_meta(|meta| {
                            if meta.input.parse::<Token![=]>().is_ok() {
                                if meta.path.is_ident("deserialize") {
                                    let content = meta.input.parse::<LitStr>()?;
                                    self.rename_deserialize = Some(content.value());
                                }

                                if meta.path.is_ident("serialize") {
                                    let content = meta.input.parse::<LitStr>()?;
                                    self.rename_serialize = Some(content.value());
                                }
                            }

                            Ok(())
                        })?;
                    } else if meta.input.parse::<Token![=]>().is_ok() {
                        // rename = ".."
                        let content = meta.input.parse::<LitStr>()?;
                        self.rename = Some(content.value());
                    }
                }

                if meta.path.is_ident("alias") && meta.input.parse::<Token![=]>().is_ok() {
                    // alias = ".."
                    let content = meta.input.parse::<LitStr>()?;
                    self.aliases.push(content.value());
                }

                if meta.path.is_ident("flatten") {
                    self.flatten = true;
                }

                if meta.path.is_ident("skip") {
                    self.skip = true;
                }

                Ok(())
            })?;

            Ok(())
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

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn parses_container_rename() {
            let mut ca = ContainerAttributes::default();

            ca.parse(&syn::parse_quote!(#[serde(rename = "re")]))
                .unwrap();
            assert_eq!(ca.rename, Some("re".to_string()));

            ca.parse(&syn::parse_quote!(#[serde(rename(deserialize = "de"))]))
                .unwrap();
            assert_eq!(ca.rename_deserialize, Some("de".to_string()));

            ca.parse(&syn::parse_quote!(#[serde(rename(serialize = "se"))]))
                .unwrap();
            assert_eq!(ca.rename_serialize, Some("se".to_string()));

            ca.parse(&syn::parse_quote!(#[serde(rename(deserialize = "de2", serialize = "se2"))]))
                .unwrap();
            assert_eq!(ca.rename_deserialize, Some("de2".to_string()));
            assert_eq!(ca.rename_serialize, Some("se2".to_string()));
        }

        #[test]
        fn parses_container_rename_all() {
            let mut ca = ContainerAttributes::default();

            ca.parse(&syn::parse_quote!(#[serde(rename_all = "UPPERCASE")]))
                .unwrap();
            assert!(matches!(ca.rename_all, Some(RenameAll::Uppercase)));

            ca.parse(&syn::parse_quote!(#[serde(rename_all(deserialize = "UPPERCASE"))]))
                .unwrap();
            assert!(matches!(
                ca.rename_all_deserialize,
                Some(RenameAll::Uppercase)
            ));

            ca.parse(&syn::parse_quote!(#[serde(rename_all(serialize = "UPPERCASE"))]))
                .unwrap();
            assert!(matches!(
                ca.rename_all_serialize,
                Some(RenameAll::Uppercase)
            ));

            ca.parse(&syn::parse_quote!(#[serde(rename_all(deserialize = "lowercase", serialize = "lowercase"))]))
                .unwrap();
            assert!(matches!(
                ca.rename_all_deserialize,
                Some(RenameAll::Lowercase)
            ));
            assert!(matches!(
                ca.rename_all_serialize,
                Some(RenameAll::Lowercase)
            ));
        }

        #[test]
        fn parses_container_rename_all_fields() {
            let mut ca = ContainerAttributes::default();

            ca.parse(&syn::parse_quote!(#[serde(rename_all_fields = "UPPERCASE")]))
                .unwrap();
            assert!(matches!(ca.rename_all_fields, Some(RenameAll::Uppercase)));

            ca.parse(&syn::parse_quote!(#[serde(rename_all_fields(deserialize = "UPPERCASE"))]))
                .unwrap();
            assert!(matches!(
                ca.rename_all_fields_deserialize,
                Some(RenameAll::Uppercase)
            ));

            ca.parse(&syn::parse_quote!(#[serde(rename_all_fields(serialize = "UPPERCASE"))]))
                .unwrap();
            assert!(matches!(
                ca.rename_all_fields_serialize,
                Some(RenameAll::Uppercase)
            ));

            ca.parse(&syn::parse_quote!(#[serde(rename_all_fields(deserialize = "lowercase", serialize = "lowercase"))]))
                .unwrap();
            assert!(matches!(
                ca.rename_all_fields_deserialize,
                Some(RenameAll::Lowercase)
            ));
            assert!(matches!(
                ca.rename_all_fields_serialize,
                Some(RenameAll::Lowercase)
            ));
        }

        #[test]
        fn parses_container_enum_representation() {
            let mut ca = ContainerAttributes::default();

            ca.parse(&syn::parse_quote!(#[serde(tag = "t")])).unwrap();
            if let EnumRepresentation::Internal { tag } = &ca.enum_representation {
                assert_eq!(tag, "t");
            } else {
                panic!("failed to parse tag");
            }

            ca.parse(&syn::parse_quote!(#[serde(tag = "t", content = "c")]))
                .unwrap();
            if let EnumRepresentation::Adjacent { tag, content } = &ca.enum_representation {
                assert_eq!(tag, "t");
                assert_eq!(content, "c");
            } else {
                panic!("failed to parse tag and content");
            }

            ca.parse(&syn::parse_quote!(#[serde(untagged)])).unwrap();
            assert!(matches!(
                ca.enum_representation,
                EnumRepresentation::Untagged
            ));
        }

        #[test]
        fn parses_variant_rename() {
            let mut va = VariantAttributes::default();

            va.parse(&syn::parse_quote!(#[serde(rename = "re")]))
                .unwrap();
            assert_eq!(va.rename, Some("re".to_string()));

            va.parse(&syn::parse_quote!(#[serde(rename(deserialize = "de"))]))
                .unwrap();
            assert_eq!(va.rename_deserialize, Some("de".to_string()));

            va.parse(&syn::parse_quote!(#[serde(rename(serialize = "se"))]))
                .unwrap();
            assert_eq!(va.rename_serialize, Some("se".to_string()));

            va.parse(&syn::parse_quote!(#[serde(rename(deserialize = "de2", serialize = "se2"))]))
                .unwrap();
            assert_eq!(va.rename_deserialize, Some("de2".to_string()));
            assert_eq!(va.rename_serialize, Some("se2".to_string()));
        }

        #[test]
        fn parses_variant_rename_all() {
            let mut va = VariantAttributes::default();

            va.parse(&syn::parse_quote!(#[serde(rename_all = "UPPERCASE")]))
                .unwrap();
            assert!(matches!(va.rename_all, Some(RenameAll::Uppercase)));

            va.parse(&syn::parse_quote!(#[serde(rename_all(deserialize = "UPPERCASE"))]))
                .unwrap();
            assert!(matches!(
                va.rename_all_deserialize,
                Some(RenameAll::Uppercase)
            ));

            va.parse(&syn::parse_quote!(#[serde(rename_all(serialize = "UPPERCASE"))]))
                .unwrap();
            assert!(matches!(
                va.rename_all_serialize,
                Some(RenameAll::Uppercase)
            ));

            va.parse(&syn::parse_quote!(#[serde(rename_all(deserialize = "lowercase", serialize = "lowercase"))]))
                .unwrap();
            assert!(matches!(
                va.rename_all_deserialize,
                Some(RenameAll::Lowercase)
            ));
            assert!(matches!(
                va.rename_all_serialize,
                Some(RenameAll::Lowercase)
            ));
        }

        #[test]
        fn parses_variant_aliases() {
            let mut va = VariantAttributes::default();
            va.parse(&syn::parse_quote!(#[serde(alias = "a")])).unwrap();
            assert_eq!(va.aliases[0], "a");
            va.parse(&syn::parse_quote!(#[serde(alias = "a1", alias = "a2")]))
                .unwrap();
            assert_eq!(va.aliases[1], "a1");
            assert_eq!(va.aliases[2], "a2");
        }

        #[test]
        fn parses_variant_skip() {
            let mut va = VariantAttributes::default();
            va.parse(&syn::parse_quote!(#[serde(skip)])).unwrap();
            assert!(va.skip);
        }

        #[test]
        fn parses_variant_other() {
            let mut va = VariantAttributes::default();
            va.parse(&syn::parse_quote!(#[serde(other)])).unwrap();
            assert!(va.other);
        }

        #[test]
        fn parses_field_rename() {
            let mut fa = FieldAttributes::default();

            fa.parse(&syn::parse_quote!(#[serde(rename = "re")]))
                .unwrap();
            assert_eq!(fa.rename, Some("re".to_string()));

            fa.parse(&syn::parse_quote!(#[serde(rename(deserialize = "de"))]))
                .unwrap();
            assert_eq!(fa.rename_deserialize, Some("de".to_string()));

            fa.parse(&syn::parse_quote!(#[serde(rename(serialize = "se"))]))
                .unwrap();
            assert_eq!(fa.rename_serialize, Some("se".to_string()));

            fa.parse(&syn::parse_quote!(#[serde(rename(deserialize = "de2", serialize = "se2"))]))
                .unwrap();
            assert_eq!(fa.rename_deserialize, Some("de2".to_string()));
            assert_eq!(fa.rename_serialize, Some("se2".to_string()));
        }

        #[test]
        fn parses_field_aliases() {
            let mut fa = FieldAttributes::default();
            fa.parse(&syn::parse_quote!(#[serde(alias = "a")])).unwrap();
            assert_eq!(fa.aliases[0], "a");
            fa.parse(&syn::parse_quote!(#[serde(alias = "a1", alias = "a2")]))
                .unwrap();
            assert_eq!(fa.aliases[1], "a1");
            assert_eq!(fa.aliases[2], "a2");
        }

        #[test]
        fn parses_field_flatten() {
            let mut fa = FieldAttributes::default();
            fa.parse(&syn::parse_quote!(#[serde(flatten)])).unwrap();
            assert!(fa.flatten);
        }

        #[test]
        fn parses_field_skip() {
            let mut fa = FieldAttributes::default();
            fa.parse(&syn::parse_quote!(#[serde(skip)])).unwrap();
            assert!(fa.skip);
        }
    }
}
