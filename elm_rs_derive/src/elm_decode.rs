//! Derive macro for ElmDecode.

use super::{EnumVariantKind, Intermediate, TypeInfo};
#[cfg(feature = "serde")]
use crate::attributes::serde::EnumRepresentation;
use crate::{attributes::ContainerAttributes, EnumVariant, StructField};
use heck::ToLowerCamelCase;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Type};

pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let intermediate = match Intermediate::parse(derive_input) {
        Ok(intermediate) => intermediate,
        Err(err) => return err.to_compile_error().into(),
    };
    let token_stream = match intermediate_to_token_stream(intermediate) {
        Ok(token_stream) => token_stream,
        Err(err) => return err.to_compile_error().into(),
    };
    TokenStream::from(token_stream)
}

fn intermediate_to_token_stream(
    Intermediate {
        ident,
        elm_type,
        mut generics,
        generics_without_bounds,
        type_info,
        container_attributes,
    }: Intermediate,
) -> Result<TokenStream2, syn::Error> {
    let decoder_type = format!("{}Decoder", elm_type.to_lower_camel_case());

    let decoder = match type_info {
        TypeInfo::Unit => struct_unit(&elm_type, &decoder_type),
        TypeInfo::Newtype(ty) => struct_newtype(&elm_type, &decoder_type, &ty),
        TypeInfo::Tuple(tys) => struct_tuple(&elm_type, &decoder_type, &tys),
        TypeInfo::Struct(fields) => {
            struct_named(&elm_type, &decoder_type, &fields, &container_attributes)
        }
        TypeInfo::Enum {
            variants,
            #[cfg(feature = "serde")]
            representation,
        } => {
            #[cfg(feature = "serde")]
            let representation = match representation {
                EnumRepresentation::External => {
                    enum_external(&elm_type, &decoder_type, variants, &container_attributes)
                }
                EnumRepresentation::Internal { tag } => enum_internal(
                    &elm_type,
                    &decoder_type,
                    variants,
                    &tag,
                    &container_attributes,
                )?,
                EnumRepresentation::Adjacent { tag, content } => enum_adjacent(
                    &elm_type,
                    &decoder_type,
                    variants,
                    &tag,
                    &content,
                    &container_attributes,
                )?,
                EnumRepresentation::Untagged => {
                    enum_untagged(&elm_type, &decoder_type, variants, &container_attributes)?
                }
            };
            #[cfg(not(feature = "serde"))]
            let representation =
                enum_external(&elm_type, &decoder_type, variants, &container_attributes);
            representation
        }
    };

    for p in generics.type_params_mut() {
        p.bounds.push(syn::parse_str("::elm_rs::Elm").unwrap());
        p.bounds
            .push(syn::parse_str("::elm_rs::ElmDecode").unwrap());
    }

    let res = quote! {
        impl #generics ::elm_rs::ElmDecode for #ident #generics_without_bounds {
            fn decoder_type() -> ::std::string::String {
                ::std::convert::From::from(#decoder_type)
            }

            fn decoder_definition() -> ::std::option::Option<::std::string::String> {
                ::std::option::Option::Some(#decoder)
            }
        }
    };
    Ok(res)
}

// =======
// structs
// =======

/// #[derive(Deserialize, Serialize)]
/// struct Unit;
/// "null"
fn struct_unit(elm_type: &str, decoder_type: &str) -> TokenStream2 {
    quote! {::std::format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    Json.Decode.null {elm_type}
",
        elm_type = #elm_type,
        decoder_type = #decoder_type,
    )}
}

/// #[derive(Deserialize, Serialize)]
/// struct Newtype(i32);
/// "0"
fn struct_newtype(elm_type: &str, decoder_type: &str, ty: &Type) -> TokenStream2 {
    quote! {::std::format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    Json.Decode.map {elm_type} ({inner_decoder})
",
        elm_type = #elm_type,
        decoder_type = #decoder_type,
        inner_decoder = <#ty>::decoder_type(),
    )}
}

/// #[derive(Deserialize, Serialize)]
/// struct Tuple(i32, i32);
/// "[0,0]"
fn struct_tuple(elm_type: &str, decoder_type: &str, inner_types: &[Type]) -> TokenStream2 {
    let indices: Vec<usize> = inner_types.iter().enumerate().map(|(i, _)| i).collect();
    quote! {::std::format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    Json.Decode.succeed {elm_type}
        {decoders}
",
        elm_type = #elm_type,
        decoder_type = #decoder_type,
        decoders = (
            &[
                #(::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.index {idx} ({decoder}) |> Json.Decode.map x)",
                    idx = #indices,
                    decoder = <#inner_types>::decoder_type())
                ),*
            ]
        ).join("\n        ")
    )}
}

/// #[derive(Deserialize, Serialize)]
/// struct Struct {
///     a: i32,
/// };
/// "{\"a\":0}"
fn struct_named(
    elm_type: &str,
    decoder_type: &str,
    fields: &[StructField],
    container_attributes: &ContainerAttributes,
) -> TokenStream2 {
    let mut field_decoders = vec![];
    for field in fields {
        let ty = &field.ty;
        let field_name_decode = field.name_decode(container_attributes);
        field_decoders.push(quote!{::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.field \"{field_name_decode}\" ({decoder})))",
                field_name_decode = #field_name_decode,
                decoder = <#ty as ::elm_rs::ElmDecode>::decoder_type(),
        )});
    }
    quote! {::std::format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    Json.Decode.succeed {elm_type}
        {field_decoders}
",
        elm_type = #elm_type,
        decoder_type = #decoder_type,
        field_decoders = (
            &[
                #(#field_decoders),*
            ]
        ).join("\n        "),
    )}
}

// =====
// enums
// =====

// everything is contained under the variant field
/// #[derive(Deserialize, Serialize)]
/// enum External {
///     Unit,
///     Newtype(i32),
///     Tuple(i32, i32),
///     Struct { a: i32 },
/// }
/// "\"Unit\""
/// "{\"Newtype\":0}"
/// "{\"Tuple\":[0,0]}"
/// "{\"Struct\":{\"a\":0}}"
fn enum_external(
    elm_type: &str,
    decoder_name: &str,
    variants: Vec<EnumVariant>,
    container_attributes: &ContainerAttributes,
) -> TokenStream2 {
    let mut decoders = vec![];
    let mut constructors = vec![];
    let mut other_decoder = None;
    for variant in variants {
        #[cfg(feature = "serde")]
        if variant.serde_attributes.skip {
            continue;
        }

        let elm_name = variant.name_elm();
        let elm_name_decode = variant.name_decode(container_attributes);
        #[cfg(feature = "serde")]
        if variant.serde_attributes.other {
            other_decoder =
                Some(quote! {::std::format!("Json.Decode.succeed {}", #elm_name_decode)});
        }

        let decoder = match &variant.variant {
            EnumVariantKind::Unit => enum_variant_unit_external(&elm_name, &elm_name_decode),
            EnumVariantKind::Newtype(inner) => {
                enum_variant_newtype_external(&elm_name, &elm_name_decode, inner)
            }
            EnumVariantKind::Tuple(types) => {
                enum_variant_tuple_external(&elm_name, &elm_name_decode, types)
            }
            EnumVariantKind::Struct(fields) => {
                let (decoder, constructor) = enum_variant_struct_external(
                    &elm_name,
                    &elm_name_decode,
                    fields,
                    container_attributes,
                );
                constructors.push(constructor);
                decoder
            }
        };
        decoders.push(decoder);
    }
    if let Some(other_decoder) = other_decoder {
        decoders.push(other_decoder)
    }

    let constructors = if constructors.is_empty() {
        quote! {""}
    } else {
        quote! {::std::format!("
        let
            {}
        in", (&[
                    #(#constructors),*
            ]).join("\n            ")
        )}
    };

    quote! {::std::format!("\
{decoder_name} : Json.Decode.Decoder {elm_type}
{decoder_name} = {constructors}
    Json.Decode.oneOf
        [ {decoders}
        ]",
        decoder_name = #decoder_name,
        elm_type = #elm_type,
        constructors = #constructors,
        decoders = (
            &[
                #(#decoders),*
            ]
        ).join("\n        , ")
    )}
}

// an object with a tag field
/// #[derive(Deserialize, Serialize)]
/// #[serde(tag = "t")]
/// enum Internal {
///     Unit,
///     Struct { a: i32 },
/// }
/// "{\"t\":\"Unit\"}"
/// "{\"t\":\"Struct\",\"a\":0}"
#[cfg(feature = "serde")]
fn enum_internal(
    elm_type: &str,
    decoder_name: &str,
    variants: Vec<EnumVariant>,
    tag: &str,
    container_attributes: &ContainerAttributes,
) -> Result<TokenStream2, syn::Error> {
    let mut decoders = vec![];
    let mut constructors = vec![];
    let mut other_decoder = None;
    for variant in variants {
        #[cfg(feature = "serde")]
        if variant.serde_attributes.skip {
            continue;
        }

        let elm_name = variant.name_elm();
        let elm_name_decode = variant.name_decode(container_attributes);
        #[cfg(feature = "serde")]
        if variant.serde_attributes.other {
            other_decoder = Some(quote! {::std::format!("\
_ ->
                Json.Decode.succeed {}", #elm_name_decode)});
        }

        let decoder = match &variant.variant {
            EnumVariantKind::Unit => {
                enum_variant_unit_internal_or_adjacent(&elm_name, &elm_name_decode)
            }
            EnumVariantKind::Newtype(_) => {
                return Err(syn::Error::new(
                    variant.span,
                    "Internally tagged newtype variants are not supported by serde_json",
                ))
            }
            EnumVariantKind::Tuple(_) => {
                return Err(syn::Error::new(
                    variant.span,
                    "Internally tagged tuple variants are not supported by serde_json",
                ))
            }
            EnumVariantKind::Struct(fields) => {
                let (decoder, constructor) = enum_variant_struct_internal(
                    &elm_name,
                    &elm_name_decode,
                    fields,
                    container_attributes,
                );
                constructors.push(constructor);
                decoder
            }
        };
        decoders.push(decoder);
    }
    if let Some(other_decoder) = other_decoder {
        decoders.push(other_decoder)
    }

    let constructors = if constructors.is_empty() {
        quote! {""}
    } else {
        quote! {::std::format!("
        let
            {}
        in", (&[
                    #(#constructors),*
            ]).join("\n            ")
        )}
    };

    let decoder = quote! {::std::format!("\
{decoder_name} : Json.Decode.Decoder {elm_type}
{decoder_name} = {constructors}
    Json.Decode.field \"{tag}\" Json.Decode.string
        |> Json.Decode.andThen
            (\\tag ->
                case tag of
                    {decoders}
                    unexpected ->
                        Json.Decode.fail <| \"Unexpected variant \" ++ unexpected
            )",
        decoder_name = #decoder_name,
        elm_type = #elm_type,
        constructors = #constructors,
        tag = #tag,
        decoders = (
            &[
                #(#decoders),*
            ]
        ).join("\n                    ")
    )};

    Ok(decoder)
}

// an object with tag and content fields
/// #[derive(Deserialize, Serialize)]
/// #[serde(tag = "t", content = "c")]
/// enum Adjacent {
///     Unit,
///     Newtype(i32),
///     Tuple(i32, i32),
///     Struct { a: i32 },
/// }
/// "{\"t\":\"Unit\"}"
/// "{\"t\":\"Newtype\",\"c\":0}"
/// "{\"t\":\"Tuple\",\"c\":[0,0]}"
/// "{\"t\":\"Struct\",\"c\":{\"a\":0}}"
#[cfg(feature = "serde")]
fn enum_adjacent(
    elm_type: &str,
    decoder_name: &str,
    variants: Vec<EnumVariant>,
    tag: &str,
    content: &str,
    container_attributes: &ContainerAttributes,
) -> Result<TokenStream2, syn::Error> {
    let mut decoders = vec![];
    let mut constructors = vec![];
    let mut other_decoder = None;
    for variant in variants {
        #[cfg(feature = "serde")]
        if variant.serde_attributes.skip {
            continue;
        }

        let elm_name = variant.name_elm();
        let elm_name_decode = variant.name_decode(container_attributes);
        #[cfg(feature = "serde")]
        if variant.serde_attributes.other {
            other_decoder = Some(quote! {::std::format!("\
_ ->
                Json.Decode.succeed {}", #elm_name_decode)});
        }

        let decoder = match &variant.variant {
            EnumVariantKind::Unit => {
                enum_variant_unit_internal_or_adjacent(&elm_name, &elm_name_decode)
            }
            EnumVariantKind::Newtype(inner) => {
                enum_variant_newtype_adjacent(content, &elm_name, &elm_name_decode, inner)
            }
            EnumVariantKind::Tuple(types) => {
                enum_variant_tuple_adjacent(content, &elm_name, &elm_name_decode, types)
            }
            EnumVariantKind::Struct(fields) => {
                let (decoder, constructor) = enum_variant_struct_adjacent(
                    content,
                    &elm_name,
                    &elm_name_decode,
                    fields,
                    container_attributes,
                );
                constructors.push(constructor);
                decoder
            }
        };
        decoders.push(decoder);
    }
    if let Some(other_decoder) = other_decoder {
        decoders.push(other_decoder)
    }

    let constructors = if constructors.is_empty() {
        quote! {""}
    } else {
        quote! {::std::format!("
        let
            {}
        in", (&[
                    #(#constructors),*
            ]).join("\n            ")
        )}
    };

    let decoder = quote! {::std::format!("\
{decoder_name} : Json.Decode.Decoder {elm_type}
{decoder_name} = {constructors}
    Json.Decode.field \"{tag}\" Json.Decode.string
        |> Json.Decode.andThen
            (\\tag ->
                case tag of
                    {decoders}
                    unexpected ->
                        Json.Decode.fail <| \"Unexpected variant \" ++ unexpected
            )",
        decoder_name = #decoder_name,
        elm_type = #elm_type,
        constructors = #constructors,
        tag = #tag,
        decoders = (
            &[
                #(#decoders),*
            ]
        ).join("\n                    ")
    )};
    Ok(decoder)
}

// no tag
/// #[derive(Deserialize, Serialize)]
/// #[serde(untagged)]
/// enum Untagged {
///     Unit,
///     Newtype(i32),
///     Tuple(i32, i32),
///     Struct { a: i32 },
/// }
/// "null"
/// "0"
/// "[0,0]"
/// "{\"a\":0}"
#[cfg(feature = "serde")]
fn enum_untagged(
    elm_type: &str,
    decoder_name: &str,
    variants: Vec<EnumVariant>,
    container_attributes: &ContainerAttributes,
) -> Result<TokenStream2, syn::Error> {
    let mut decoders = vec![];
    let mut constructors = vec![];
    for variant in variants {
        #[cfg(feature = "serde")]
        if variant.serde_attributes.skip {
            continue;
        }

        let elm_name = variant.name_elm();
        let decoder = match &variant.variant {
            EnumVariantKind::Unit => enum_variant_unit_untagged(&elm_name),
            EnumVariantKind::Newtype(inner) => enum_variant_newtype_untagged(&elm_name, inner),
            EnumVariantKind::Tuple(types) => enum_variant_tuple_untagged(&elm_name, types),
            EnumVariantKind::Struct(fields) => {
                let (decoder, constructor) =
                    enum_variant_struct_untagged(&elm_name, fields, container_attributes);
                constructors.push(constructor);
                decoder
            }
        };
        decoders.push(decoder);
    }

    let constructors = if constructors.is_empty() {
        quote! {""}
    } else {
        quote! {::std::format!("
        let
            {}
        in", (&[
                    #(#constructors),*
            ]).join("\n            ")
        )}
    };

    let decoder = quote! {::std::format!("\
{decoder_name} : Json.Decode.Decoder {elm_type}
{decoder_name} = {constructors}
    Json.Decode.oneOf
        [ {decoders}
        ]",
        decoder_name = #decoder_name,
        elm_type = #elm_type,
        constructors = #constructors,
        decoders = (
            &[
                #(#decoders),*
            ]
        ).join("\n    , ")
    )};

    Ok(decoder)
}

// =================
// external variants
// =================

/// #[derive(Deserialize, Serialize)]
/// enum External {
///     Unit,
/// }
/// "\"Unit\""
fn enum_variant_unit_external(variant_name: &str, variant_name_decode: &str) -> TokenStream2 {
    quote! {::std::format!("\
Json.Decode.string
            |> Json.Decode.andThen
                (\\x ->
                    case x of
                        \"{enum_variant_deserialize}\" ->
                            Json.Decode.succeed {variant_name}
                        unexpected ->
                            Json.Decode.fail <| \"Unexpected variant \" ++ unexpected
                )",
        variant_name = #variant_name,
        enum_variant_deserialize = #variant_name_decode
    )}
}

/// #[derive(Deserialize, Serialize)]
/// enum External {
///     Newtype(i32),
/// }
/// "{\"Newtype\":0}"
fn enum_variant_newtype_external(
    variant_name: &str,
    variant_name_decode: &str,
    inner_type: &TokenStream2,
) -> TokenStream2 {
    quote! {::std::format!("\
    Json.Decode.map {enum_variant} (Json.Decode.field \"{enum_variant_deserialize}\" ({decoder}))",
        enum_variant = #variant_name,
        enum_variant_deserialize = #variant_name_decode,
        decoder = <#inner_type as ::elm_rs::ElmDecode>::decoder_type(),
    )}
}

/// #[derive(Deserialize, Serialize)]
/// enum External {
///     Tuple(i32, i32),
/// }
/// "{\"Tuple\":[0,0]}"
fn enum_variant_tuple_external(
    variant_name: &str,
    variant_name_decode: &str,
    tuple_types: &[TokenStream2],
) -> TokenStream2 {
    let idx: Vec<usize> = (0..tuple_types.len()).collect();

    quote! {::std::format!("\
    Json.Decode.field \"{variant_name_decode}\" (Json.Decode.succeed {variant_name} {decoders})",
        variant_name = #variant_name,
        variant_name_decode = #variant_name_decode,
        decoders = (
            &[
                #(::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.index {} ({}) |> Json.Decode.map x)",
                    #idx,
                    <#tuple_types as ::elm_rs::ElmDecode>::decoder_type()
                )),*
            ]
        ).join(" ")
    )}
}

/// #[derive(Deserialize, Serialize)]
/// enum External {
///     Struct { a: i32 },
/// }
/// "{\"Struct\":{\"a\":0}}"
fn enum_variant_struct_external(
    variant_name: &str,
    variant_name_decode: &str,
    fields: &[StructField],
    container_attributes: &ContainerAttributes,
) -> (TokenStream2, TokenStream2) {
    let (field_names, tys): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|field| (field.name_elm(), &field.ty))
        .unzip();
    let field_names_deserialize = fields
        .iter()
        .map(|field| field.name_decode(container_attributes));

    let constructor = constructor(variant_name, &field_names);

    let decoder = quote! {::std::format!("\
    Json.Decode.field \"{variant_name_decode}\" (Json.Decode.succeed elmRsConstruct{variant_name} {decoders})",
            variant_name = #variant_name,
            variant_name_decode = #variant_name_decode,
            decoders = (
                &[
                    #(::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.field \"{}\" ({})))",
                        #field_names_deserialize,
                        <#tys as ::elm_rs::ElmDecode>::decoder_type()
                    )),*
                ]
            ).join(" "),
    )};

    (decoder, constructor)
}

// =================
// internal variants
// =================

/// #[derive(Deserialize, Serialize)]
/// #[serde(tag = "t")]
/// enum Internal {
///     Unit,
/// }
/// #[derive(Deserialize, Serialize)]
/// #[serde(tag = "t", content = "c")]
/// enum Adjacent {
///     Unit,
/// }
/// "{\"t\":\"Unit\"}"
#[cfg(feature = "serde")]
fn enum_variant_unit_internal_or_adjacent(
    variant_name: &str,
    variant_name_decode: &str,
) -> TokenStream2 {
    quote! { format!("\
\"{variant_name_decode}\" ->
                        Json.Decode.succeed {variant_name}",
        variant_name = #variant_name,
        variant_name_decode = #variant_name_decode,
    )}
}

/// #[derive(Deserialize, Serialize)]
/// #[serde(tag = "t")]
/// enum Internal {
///     Struct { a: i32 },
/// }
/// "{\"t\":\"Struct\",\"a\":0}"
#[cfg(feature = "serde")]
fn enum_variant_struct_internal(
    variant_name: &str,
    variant_name_decode: &str,
    fields: &[StructField],
    container_attributes: &ContainerAttributes,
) -> (TokenStream2, TokenStream2) {
    let (field_names, tys): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|field| (field.name_elm(), &field.ty))
        .unzip();
    let field_names_deserialize = fields
        .iter()
        .map(|field| field.name_decode(container_attributes));

    let constructor = constructor(variant_name, &field_names);

    let decoder = quote! {::std::format!("\
                    \"{variant_name_decode}\" ->
                        Json.Decode.succeed elmRsConstruct{variant_name} {decoders}",
            variant_name = #variant_name,
            variant_name_decode = #variant_name_decode,
            decoders = (
                &[
                    #(::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.field \"{}\" ({})))",
                        #field_names_deserialize,
                        <#tys as ::elm_rs::ElmDecode>::decoder_type()
                    )),*
                ]
            ).join(" "),
    )};

    (decoder, constructor)
}

// #################
// adjacent variants
// #################

/// #[derive(Deserialize, Serialize)]
/// #[serde(tag = "t", content = "c")]
/// enum Adjacent {
///     Newtype(i32),
/// }
/// "{\"t\":\"Newtype\",\"c\":0}"
#[cfg(feature = "serde")]
fn enum_variant_newtype_adjacent(
    content: &str,
    variant_name: &str,
    variant_name_decode: &str,
    inner_type: &TokenStream2,
) -> TokenStream2 {
    quote! {::std::format!("\
\"{variant_name_decode}\" ->
                        Json.Decode.map {variant_name} (Json.Decode.field \"{content}\" ({decoder}))",
        variant_name = #variant_name,
        variant_name_decode = #variant_name_decode,
        content = #content,
        decoder = <#inner_type as ::elm_rs::ElmDecode>::decoder_type(),
    )}
}

/// #[derive(Deserialize, Serialize)]
/// #[serde(tag = "t", content = "c")]
/// enum Adjacent {
///     Tuple(i32, i32),
/// }
/// "{\"t\":\"Tuple\",\"c\":[0,0]}"
#[cfg(feature = "serde")]
fn enum_variant_tuple_adjacent(
    content: &str,
    variant_name: &str,
    variant_name_decode: &str,
    tuple_types: &[TokenStream2],
) -> TokenStream2 {
    let idx: Vec<usize> = (0..tuple_types.len()).collect();

    quote! {::std::format!("\
\"{variant_name_decode}\" ->
                        Json.Decode.field \"{content}\" (Json.Decode.succeed {variant_name} {decoders})",
        variant_name = #variant_name,
        variant_name_decode = #variant_name_decode,
        content = #content,
        decoders = (
            &[
                #(::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.index {} ({}) |> Json.Decode.map x)",
                    #idx,
                    <#tuple_types as ::elm_rs::ElmDecode>::decoder_type())
                ),*
            ]
        ).join(" "),
    )}
}

/// #[derive(Deserialize, Serialize)]
/// #[serde(tag = "t", content = "c")]
/// enum Adjacent {
///     Struct { a: i32 },
/// }
/// "{\"t\":\"Struct\",\"c\":{\"a\":0}}"
#[cfg(feature = "serde")]
fn enum_variant_struct_adjacent(
    content: &str,
    variant_name: &str,
    variant_name_decode: &str,
    fields: &[StructField],
    container_attributes: &ContainerAttributes,
) -> (TokenStream2, TokenStream2) {
    let field_names = fields
        .iter()
        .map(|field| field.name_elm())
        .collect::<Vec<_>>();
    let constructor = constructor(variant_name, &field_names);

    let mut field_decoders = vec![];
    for field in fields {
        let ty = &field.ty;
        let field_name_decode = field.name_decode(container_attributes);
        field_decoders.push(quote!{::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.field \"{field_name_decode}\" ({decoder})))",
                field_name_decode = #field_name_decode,
                decoder = <#ty as ::elm_rs::ElmDecode>::decoder_type(),
        )});
    }
    let decoder = quote! {::std::format!("\
\"{variant_name_decode}\" ->
                        Json.Decode.field \"{content}\" (Json.Decode.succeed elmRsConstruct{variant_name} {field_decoders})",
        variant_name = #variant_name,
        variant_name_decode = #variant_name_decode,
        content = #content,
        field_decoders = (
            &[
                #(#field_decoders),*
            ]
        ).join("\n        "),
    )};

    (decoder, constructor)
}

// #################
// untagged variants
// #################

/// #[derive(Deserialize, Serialize)]
/// #[serde(untagged)]
/// enum Untagged {
///     Unit,
/// }
/// "null"
#[cfg(feature = "serde")]
fn enum_variant_unit_untagged(variant_name: &str) -> TokenStream2 {
    quote! {::std::format!("\
    Json.Decode.null {variant_name}",
        variant_name = #variant_name
    )}
}

/// #[derive(Deserialize, Serialize)]
/// #[serde(untagged)]
/// enum Untagged {
///     Newtype(i32),
/// }
/// "0"
#[cfg(feature = "serde")]
fn enum_variant_newtype_untagged(variant_name: &str, inner: &TokenStream2) -> TokenStream2 {
    quote! {::std::format!("\
    Json.Decode.map {variant_name} ({decoder})",
        variant_name = #variant_name,
        decoder = <#inner as ::elm_rs::ElmDecode>::decoder_type(),
    )}
}

/// #[derive(Deserialize, Serialize)]
/// #[serde(untagged)]
/// enum Untagged {
///     Tuple(i32, i32),
/// }
/// "[0,0]"
#[cfg(feature = "serde")]
fn enum_variant_tuple_untagged(variant_name: &str, tuple_types: &[TokenStream2]) -> TokenStream2 {
    let idx: Vec<usize> = (0..tuple_types.len()).collect();
    quote! {::std::format!("\
    Json.Decode.succeed {enum_variant} {decoders}",
        enum_variant = #variant_name,
        decoders = (
            &[
                #(::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.index {} ({}) |> Json.Decode.map x)",
                    #idx,
                    <#tuple_types as ::elm_rs::ElmDecode>::decoder_type()
                )),*
            ]
        ).join(" ")
    )}
}

/// #[derive(Deserialize, Serialize)]
/// #[serde(untagged)]
/// enum Untagged {
///     Struct { a: i32 },
/// }
/// "{\"a\":0}"
#[cfg(feature = "serde")]
fn enum_variant_struct_untagged(
    variant_name: &str,
    fields: &[StructField],
    container_attributes: &ContainerAttributes,
) -> (TokenStream2, TokenStream2) {
    let (field_names, tys): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|field| (field.name_elm(), &field.ty))
        .unzip();
    let field_names_deserialize = fields
        .iter()
        .map(|field| field.name_decode(container_attributes));
    let constructor = constructor(variant_name, &field_names);
    let decoder = quote! {::std::format!("\
    Json.Decode.succeed elmRsConstruct{variant_name} {decoders}",
            variant_name = #variant_name,
            decoders = (
                &[
                    #(::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.field \"{}\" ({})))",
                        #field_names_deserialize,
                        <#tys as ::elm_rs::ElmDecode>::decoder_type()
                    )),*
                ]
            ).join(" "),
    )};

    (decoder, constructor)
}

// #######
// helpers
// #######

fn constructor(variant_name: &str, field_names: &[String]) -> TokenStream2 {
    quote! {::std::format!("\
elmRsConstruct{enum_variant} {fields} =
                        {enum_variant} {{ {setters} }}",
        enum_variant = #variant_name,
        fields = (
            &[
                #(::std::format!("{}",
                    #field_names
                )),*
            ]
        ).join(" "),
        setters = (
            &[
                #(::std::format!("{0} = {0}",
                    #field_names
                )),*
            ]
        ).join(", "),
    )}
}
