//! Derive macros for ElmJson.

use super::{EnumVariantKind, Intermediate, TypeInfo};
use crate::{EnumRepresentation, EnumVariant, StructField};
use heck::ToLowerCamelCase;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Type};

pub fn derive_elm_json(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let intermediate = match super::derive_input_to_intermediate(derive_input) {
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
    }: Intermediate,
) -> Result<TokenStream2, syn::Error> {
    let decoder_type = format!("{}Decoder", elm_type.to_lower_camel_case());
    let encoder_type = format!("{}Encoder", elm_type.to_lower_camel_case());

    let Coder { encoder, decoder } = match type_info {
        TypeInfo::Unit => struct_unit(&elm_type, &encoder_type, &decoder_type),
        TypeInfo::Newtype(ty) => struct_newtype(&elm_type, &encoder_type, &decoder_type, &ty),
        TypeInfo::Tuple(tys) => struct_tuple(&elm_type, &encoder_type, &decoder_type, &tys),
        TypeInfo::Struct(fields) => struct_named(&elm_type, &encoder_type, &decoder_type, &fields),
        TypeInfo::Enum {
            representation,
            variants,
        } => match representation {
            EnumRepresentation::External => {
                enum_external(&elm_type, &encoder_type, &decoder_type, variants)
            }
            EnumRepresentation::Internal { tag } => {
                enum_internal(&elm_type, &encoder_type, &decoder_type, variants, &tag)?
            }
            EnumRepresentation::Adjacent { tag, content } => enum_adjacent(
                &elm_type,
                &encoder_type,
                &decoder_type,
                variants,
                &tag,
                &content,
            )?,
            EnumRepresentation::Untagged => {
                enum_untagged(&elm_type, &encoder_type, &decoder_type, variants)?
            }
        },
    };

    for p in generics.type_params_mut() {
        p.bounds.push(syn::parse_str("::jalava::Elm").unwrap());
        p.bounds.push(syn::parse_str("::jalava::ElmJson").unwrap());
    }

    let res = quote! {
        impl #generics ::jalava::ElmJson for #ident #generics_without_bounds {
            fn decoder_type() -> ::std::string::String {
                ::std::convert::From::from(#decoder_type)
            }

            fn decoder_definition() -> ::std::option::Option<::std::string::String> {
                ::std::option::Option::Some(#decoder)
            }

            fn encoder_type() -> ::std::string::String {
                ::std::convert::From::from(#encoder_type)
            }

            fn encoder_definition() -> ::std::option::Option<::std::string::String> {
                ::std::option::Option::Some(#encoder)
            }
        }
    };
    Ok(res)
}

// =======
// structs
// =======

struct Coder {
    encoder: TokenStream2,
    decoder: TokenStream2,
}

/// #[derive(Deserialize, Serialize)]
/// struct Unit;
/// "null"
fn struct_unit(elm_type: &str, encoder_type: &str, decoder_type: &str) -> Coder {
    let encoder = quote! {::std::format!("\
{encoder_type} : {elm_type} -> Json.Encode.Value
{encoder_type} _ =
    Json.Encode.null
",
        elm_type = #elm_type,
        encoder_type = #encoder_type,
    )};

    let decoder = quote! {::std::format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    Json.Decode.null {elm_type}
",
        elm_type = #elm_type,
        decoder_type = #decoder_type,
    )};

    Coder { encoder, decoder }
}

/// #[derive(Deserialize, Serialize)]
/// struct Newtype(i32);
/// "0"
fn struct_newtype(elm_type: &str, encoder_type: &str, decoder_type: &str, ty: &Type) -> Coder {
    let encoder = quote! {::std::format!("\
{encoder_type} : {elm_type} -> Json.Encode.Value
{encoder_type} ({elm_type} inner) =
    ({inner_encoder}) inner
",
        elm_type = #elm_type,
        encoder_type = #encoder_type,
        inner_encoder = <#ty>::encoder_type(),
    )};

    let decoder = quote! {::std::format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    Json.Decode.map {elm_type} ({inner_decoder})
",
        elm_type = #elm_type,
        decoder_type = #decoder_type,
        inner_decoder = <#ty>::decoder_type(),
    )};

    Coder { encoder, decoder }
}

/// #[derive(Deserialize, Serialize)]
/// struct Tuple(i32, i32);
/// "[0,0]"
fn struct_tuple(
    elm_type: &str,
    encoder_type: &str,
    decoder_type: &str,
    inner_types: &[Type],
) -> Coder {
    let indices: Vec<usize> = inner_types.iter().enumerate().map(|(i, _)| i).collect();

    let encoder = quote! {::std::format!("\
{encoder_type} : {elm_type} -> Json.Encode.Value
{encoder_type} ({elm_type} {params}) =
    Json.Encode.list identity
        [ {encoders}
        ]
",
        elm_type = #elm_type,
        encoder_type = #encoder_type,
        params = (&[#(::std::format!("t{idx}",
                idx = #indices)),*
            ]
        ).join(" "),
        encoders = (
            &[
                #(::std::format!("({encoder}) t{idx}",
                    encoder = <#inner_types>::encoder_type(),
                    idx = #indices)
                ),*
            ]
        ).join("\n        , "),
    )};

    let decoder = quote! {::std::format!("\
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
    )};

    Coder { encoder, decoder }
}

/// #[derive(Deserialize, Serialize)]
/// struct Struct {
///     a: i32,
/// };
/// "{\"a\":0}"
fn struct_named(
    elm_type: &str,
    encoder_type: &str,
    decoder_type: &str,
    fields: &[StructField],
) -> Coder {
    let mut field_decoders = vec![];
    for field in fields {
        let ty = &field.ty;
        let field_name_deserialize = field.name_deserialize();
        field_decoders.push(quote!{::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.oneOf [ Json.Decode.field \"{field_name_deserialize}\" ({decoder}) ]))",
                field_name_deserialize = #field_name_deserialize,
                decoder = <#ty as ::jalava::ElmJson>::decoder_type(),
        )});
    }
    let decoder = quote! {::std::format!("\
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
    )};

    let mut field_encoders = vec![];
    for field in fields {
        let field_name = field.name_elm();
        let field_name_serialize = field.name_serialize();
        let ty = &field.ty;
        field_encoders.push(
            quote! {::std::format!("( \"{field_name_serialize}\", ({encoder}) struct.{field_name} )",
            field_name_serialize = #field_name_serialize,
            encoder = <#ty as ::jalava::ElmJson>::encoder_type(),
            field_name = #field_name)},
        )
    }
    let encoder = quote! {::std::format!("\
{encoder_type} : {elm_type} -> Json.Encode.Value
{encoder_type} struct =
    Json.Encode.object
        [ {fields}
        ]
",
        elm_type = #elm_type,
        encoder_type = #encoder_type,
        fields = (
            &[
                #(#field_encoders),*
            ]
        ).join("\n        , "),
    )};

    Coder { encoder, decoder }
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
    encoder_name: &str,
    decoder_name: &str,
    variants: Vec<EnumVariant>,
) -> Coder {
    let mut encoders = vec![];
    let mut decoders = vec![];
    let mut constructors = vec![];
    let mut other_decoder = None;
    for variant in variants {
        if variant.skip {
            continue;
        }

        let elm_name = variant.name_elm();
        let elm_name_serialize = variant.name_serialize();
        let elm_name_deserialize = variant.name_deserialize();
        if variant.other {
            other_decoder =
                Some(quote! {::std::format!("Json.Decode.succeed {}", #elm_name_deserialize)});
        }

        let coder = match &variant.variant {
            EnumVariantKind::Unit => {
                enum_variant_unit_external(&elm_name, &elm_name_serialize, &elm_name_deserialize)
            }
            EnumVariantKind::Newtype(inner) => enum_variant_newtype_external(
                &elm_name,
                &elm_name_serialize,
                &elm_name_deserialize,
                &inner,
            ),
            EnumVariantKind::Tuple(types) => enum_variant_tuple_external(
                &elm_name,
                &elm_name_serialize,
                &elm_name_deserialize,
                types,
            ),
            EnumVariantKind::Struct(fields) => {
                let (coder, constructor) = enum_variant_struct_external(
                    &elm_name,
                    &elm_name_serialize,
                    &elm_name_deserialize,
                    fields,
                );
                constructors.push(constructor);
                coder
            }
        };
        encoders.push(coder.encoder);
        decoders.push(coder.decoder);
    }
    if let Some(other_decoder) = other_decoder {
        decoders.push(other_decoder)
    }

    let encoder = quote! {::std::format!("\
{encoder_name} : {elm_type} -> Json.Encode.Value
{encoder_name} enum =
    case enum of
        {encoders}",
        encoder_name = #encoder_name,
        elm_type = #elm_type,
        encoders = (
            &[
                #(#encoders),*
            ]
        ).join("\n        ")
    )};

    let constructors = if constructors.is_empty() {
        quote! {""}
    } else {
        quote! {::std::format!("
        let
            {}
        in", (
                &[
                    #(#constructors),*
                ]
            ).join("\n        ")
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
        ).join("\n        , ")
    )};

    Coder { encoder, decoder }
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
fn enum_internal(
    elm_type: &str,
    encoder_name: &str,
    decoder_name: &str,
    variants: Vec<EnumVariant>,
    tag: &str,
) -> Result<Coder, syn::Error> {
    let mut encoders = vec![];
    let mut decoders = vec![];
    let mut constructors = vec![];
    let mut other_decoder = None;
    for variant in variants {
        if variant.skip {
            continue;
        }

        let elm_name = variant.name_elm();
        let elm_name_serialize = variant.name_serialize();
        let elm_name_deserialize = variant.name_deserialize();
        if variant.other {
            other_decoder = Some(quote! {::std::format!("\
_ ->
                Json.Decode.succeed {}", #elm_name_deserialize)});
        }

        let coder = match &variant.variant {
            EnumVariantKind::Unit => enum_variant_unit_internal_or_adjacent(
                tag,
                &elm_name,
                &elm_name_serialize,
                &elm_name_deserialize,
            ),
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
                let (coder, constructor) = enum_variant_struct_internal(
                    tag,
                    &elm_name,
                    &elm_name_serialize,
                    &elm_name_deserialize,
                    fields,
                );
                constructors.push(constructor);
                coder
            }
        };
        encoders.push(coder.encoder);
        decoders.push(coder.decoder);
    }
    if let Some(other_decoder) = other_decoder {
        decoders.push(other_decoder)
    }

    let encoder = quote! {::std::format!("\
{encoder_name} : {elm_type} -> Json.Encode.Value
{encoder_name} enum =
    case enum of
        {encoders}",
        encoder_name = #encoder_name,
        elm_type = #elm_type,
        encoders = (&[
                #(#encoders),*
        ]).join("\n        ")
    )};

    let constructors = if constructors.is_empty() {
        quote! {""}
    } else {
        quote! {::std::format!("
        let
            {}
        in", (&[
                    #(#constructors),*
            ]).join("\n        ")
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

    Ok(Coder { encoder, decoder })
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
fn enum_adjacent(
    elm_type: &str,
    encoder_name: &str,
    decoder_name: &str,
    variants: Vec<EnumVariant>,
    tag: &str,
    content: &str,
) -> Result<Coder, syn::Error> {
    let mut encoders = vec![];
    let mut decoders = vec![];
    let mut constructors = vec![];
    let mut other_decoder = None;
    for variant in variants {
        if variant.skip {
            continue;
        }

        let elm_name = variant.name_elm();
        let elm_name_serialize = variant.name_serialize();
        let elm_name_deserialize = variant.name_deserialize();
        if variant.other {
            other_decoder = Some(quote! {::std::format!("\
_ ->
                Json.Decode.succeed {}", #elm_name_deserialize)});
        }

        let coder = match &variant.variant {
            EnumVariantKind::Unit => enum_variant_unit_internal_or_adjacent(
                tag,
                &elm_name,
                &elm_name_serialize,
                &elm_name_deserialize,
            ),
            EnumVariantKind::Newtype(inner) => enum_variant_newtype_adjacent(
                tag,
                content,
                &elm_name,
                &elm_name_serialize,
                &elm_name_deserialize,
                inner,
            ),
            EnumVariantKind::Tuple(types) => enum_variant_tuple_adjacent(
                tag,
                content,
                &elm_name,
                &elm_name_serialize,
                &elm_name_deserialize,
                &types,
            ),
            EnumVariantKind::Struct(fields) => {
                let (coder, constructor) = enum_variant_struct_adjacent(
                    tag,
                    content,
                    &elm_name,
                    &elm_name_serialize,
                    &elm_name_deserialize,
                    &fields,
                );
                constructors.push(constructor);
                coder
            }
        };
        encoders.push(coder.encoder);
        decoders.push(coder.decoder);
    }
    if let Some(other_decoder) = other_decoder {
        decoders.push(other_decoder)
    }

    let encoder = quote! {::std::format!("\
{encoder_name} : {elm_type} -> Json.Encode.Value
{encoder_name} enum =
    case enum of
        {encoders}",
        encoder_name = #encoder_name,
        elm_type = #elm_type,
        encoders = (&[
                #(#encoders),*
        ]).join("\n        ")
    )};

    let constructors = if constructors.is_empty() {
        quote! {""}
    } else {
        quote! {::std::format!("
        let
            {}
        in", (&[
                    #(#constructors),*
            ]).join("\n        ")
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

    Ok(Coder { encoder, decoder })
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
fn enum_untagged(
    elm_type: &str,
    encoder_name: &str,
    decoder_name: &str,
    variants: Vec<EnumVariant>,
) -> Result<Coder, syn::Error> {
    let mut encoders = vec![];
    let mut decoders = vec![];
    let mut constructors = vec![];
    for variant in variants {
        if variant.skip {
            continue;
        }

        let elm_name = variant.name_elm();
        let coder = match &variant.variant {
            EnumVariantKind::Unit => enum_variant_unit_untagged(&elm_name),
            EnumVariantKind::Newtype(inner) => enum_variant_newtype_untagged(&elm_name, inner),
            EnumVariantKind::Tuple(types) => enum_variant_tuple_untagged(&elm_name, types),
            EnumVariantKind::Struct(fields) => {
                let (coder, constructor) = enum_variant_struct_untagged(&elm_name, fields);
                constructors.push(constructor);
                coder
            }
        };
        encoders.push(coder.encoder);
        decoders.push(coder.decoder);
    }

    let encoder = quote! {::std::format!("\
{encoder_name} : {elm_type} -> Json.Encode.Value
{encoder_name} enum =
    case enum of
        {encoders}",
        encoder_name = #encoder_name,
        elm_type = #elm_type,
        encoders = (
            &[
                #(#encoders),*
            ]
        ).join("\n        ")
    )};

    let constructors = if constructors.is_empty() {
        quote! {""}
    } else {
        quote! {::std::format!("
        let
            {}
        in", (
                &[
                    #(#constructors),*
                ]
            ).join("\n        ")
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

    Ok(Coder { encoder, decoder })
}

// =================
// external variants
// =================

/// #[derive(Deserialize, Serialize)]
/// enum External {
///     Unit,
/// }
/// "\"Unit\""
fn enum_variant_unit_external(
    variant_name: &str,
    variant_name_serialize: &str,
    variant_name_deserialize: &str,
) -> Coder {
    let encoder = quote! {::std::format!("\
    {variant_name} ->
            Json.Encode.string \"{variant_name_serialize}\"",
        variant_name = #variant_name,
        variant_name_serialize = #variant_name_serialize,
    )};

    let decoder = quote! {::std::format!("\
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
        enum_variant_deserialize = #variant_name_deserialize
    )};

    Coder { encoder, decoder }
}

/// #[derive(Deserialize, Serialize)]
/// enum External {
///     Newtype(i32),
/// }
/// "{\"Newtype\":0}"
fn enum_variant_newtype_external(
    variant_name: &str,
    variant_name_serialize: &str,
    variant_name_deserialize: &str,
    inner_type: &Type,
) -> Coder {
    let encoder = quote! { format!("\
{variant_name} inner ->
            Json.Encode.object [ ( \"{variant_name_serialize}\", {encoder} inner ) ]",
        variant_name = #variant_name,
        variant_name_serialize = #variant_name_serialize,
        encoder = <#inner_type as ::jalava::ElmJson>::encoder_type(),
    )};

    let decoder = quote! {::std::format!("\
    Json.Decode.map {enum_variant} (Json.Decode.field \"{enum_variant_deserialize}\" ({decoder}))",
        enum_variant = #variant_name,
        enum_variant_deserialize = #variant_name_deserialize,
        decoder = <#inner_type as ::jalava::ElmJson>::decoder_type(),
    )};

    Coder { encoder, decoder }
}

/// #[derive(Deserialize, Serialize)]
/// enum External {
///     Tuple(i32, i32),
/// }
/// "{\"Tuple\":[0,0]}"
fn enum_variant_tuple_external(
    variant_name: &str,
    variant_name_serialize: &str,
    variant_name_deserialize: &str,
    tuple_types: &[Type],
) -> Coder {
    let idx: Vec<usize> = (0..tuple_types.len()).collect();
    let encoder = quote! {::std::format!("\
{variant_name} {fields} ->
            Json.Encode.object [ ( \"{variant_name_serialize}\", Json.Encode.list identity [ {encoders} ] ) ]",
        variant_name = #variant_name,
        fields = (
            &[
                #(::std::format!("t{}", #idx)
                ),*
            ]
        ).join(" "),
        variant_name_serialize = #variant_name_serialize,
        encoders = (
            &[
                #(::std::format!("{} t{}",
                    <#tuple_types as ::jalava::ElmJson>::encoder_type(),
                    #idx
                )),*
            ]
        ).join(", "),
    )};

    let decoder = quote! {::std::format!("\
    Json.Decode.field \"{variant_name_deserialize}\" (Json.Decode.succeed {variant_name} {decoders})",
        variant_name = #variant_name,
        variant_name_deserialize = #variant_name_deserialize,
        decoders = (
            &[
                #(::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.index {} ({}) |> Json.Decode.map x)",
                    #idx,
                    <#tuple_types as ::jalava::ElmJson>::decoder_type()
                )),*
            ]
        ).join(" ")
    )};

    Coder { encoder, decoder }
}

/// #[derive(Deserialize, Serialize)]
/// enum External {
///     Struct { a: i32 },
/// }
/// "{\"Struct\":{\"a\":0}}"
fn enum_variant_struct_external(
    variant_name: &str,
    variant_name_serialize: &str,
    variant_name_deserialize: &str,
    fields: &[StructField],
) -> (Coder, TokenStream2) {
    let (field_names, tys): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|field| (field.name_elm(), &field.ty))
        .unzip();
    let field_names_deserialize = fields.iter().map(|field| field.name_deserialize());
    let field_names_serialize = fields.iter().map(|field| field.name_serialize());

    let constructor = constructor(variant_name, &field_names);

    let encoder = quote! {::std::format!("\
{variant_name} {{ {fields} }} ->
            Json.Encode.object [ ( \"{variant_name_serialize}\", Json.Encode.object [ {encoders} ] ) ]",
        variant_name = #variant_name,
        variant_name_serialize = #variant_name_serialize,
        fields =  (
            &[
                #(::std::format!("{}", #field_names,
                )),*
            ]
        ).join(", "),
        encoders = (
            &[
                #(::std::format!("( \"{}\", {} {} )",
                    #field_names_serialize,
                    <#tys as ::jalava::ElmJson>::encoder_type(),
                    #field_names,
                )),*
            ]
        ).join(", ")
    )};

    let decoder = quote! {::std::format!("\
    Json.Decode.field \"{variant_name_deserialize}\" (Json.Decode.succeed construct{variant_name} {decoders})",
            variant_name = #variant_name,
            variant_name_deserialize = #variant_name_deserialize,
            decoders = (
                &[
                    #(::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.field \"{}\" ({})))",
                        #field_names_deserialize,
                        <#tys as ::jalava::ElmJson>::decoder_type()
                    )),*
                ]
            ).join(" "),
    )};

    (Coder { encoder, decoder }, constructor)
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
fn enum_variant_unit_internal_or_adjacent(
    tag: &str,
    variant_name: &str,
    variant_name_serialize: &str,
    variant_name_deserialize: &str,
) -> Coder {
    let encoder = quote! {::std::format!("\
{variant_name} ->
            Json.Encode.object [ ( \"{tag}\", Json.Encode.string \"{variant_name_serialize}\" ) ]",
        variant_name = #variant_name,
        tag = #tag,
        variant_name_serialize = #variant_name_serialize,
    )};

    let decoder = quote! { format!("\
\"{variant_name_deserialize}\" ->
                        Json.Decode.succeed {variant_name}",
        variant_name = #variant_name,
        variant_name_deserialize = #variant_name_deserialize,
    )};

    Coder { encoder, decoder }
}

/// #[derive(Deserialize, Serialize)]
/// #[serde(tag = "t")]
/// enum Internal {
///     Struct { a: i32 },
/// }
/// "{\"t\":\"Struct\",\"a\":0}"
fn enum_variant_struct_internal(
    tag: &str,
    variant_name: &str,
    variant_name_serialize: &str,
    variant_name_deserialize: &str,
    fields: &[StructField],
) -> (Coder, TokenStream2) {
    let (field_names, tys): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|field| (field.name_elm(), &field.ty))
        .unzip();
    let field_names_deserialize = fields.iter().map(|field| field.name_deserialize());
    let field_names_serialize = fields.iter().map(|field| field.name_serialize());

    let constructor = constructor(variant_name, &field_names);

    let encoder = quote! {::std::format!("\
{variant_name} {{ {fields} }} ->
            Json.Encode.object [ ( \"{tag}\", Json.Encode.string \"{variant_name_serialize}\" ), {encoders} ]",
        variant_name = #variant_name,
        fields = (&[
                #(#field_names),*
        ]).join(", "),
        tag = #tag,
        variant_name_serialize = #variant_name_serialize,
        encoders = (
            &[
                #(::std::format!("( \"{}\", {} {} )",
                    #field_names_serialize,
                    <#tys>::encoder_type(),
                    #field_names,
                )),*
            ]
        ).join(", "),
    )};

    let decoder = quote! {::std::format!("\
                    \"{variant_name_deserialize}\" ->
                        Json.Decode.succeed construct{variant_name} {decoders}",
            variant_name = #variant_name,
            variant_name_deserialize = #variant_name_deserialize,
            decoders = (
                &[
                    #(::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.field \"{}\" ({})))",
                        #field_names_deserialize,
                        <#tys as ::jalava::ElmJson>::decoder_type()
                    )),*
                ]
            ).join(" "),
    )};

    (Coder { encoder, decoder }, constructor)
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
fn enum_variant_newtype_adjacent(
    tag: &str,
    content: &str,
    variant_name: &str,
    variant_name_serialize: &str,
    variant_name_deserialize: &str,
    inner_type: &Type,
) -> Coder {
    let encoder = quote! { format!("\
    {variant_name} inner ->
            Json.Encode.object [ ( \"{tag}\", Json.Encode.string \"{variant_name_serialize}\"), ( \"{content}\", {encoder} inner ) ]",
        variant_name = #variant_name,
        tag = #tag,
        content = #content,
        variant_name_serialize = #variant_name_serialize,
        encoder = <#inner_type as ::jalava::ElmJson>::encoder_type(),
    )};

    let decoder = quote! {::std::format!("\
\"{variant_name_deserialize}\" ->
                        Json.Decode.map {variant_name} (Json.Decode.field \"{content}\" ({decoder}))",
        variant_name = #variant_name,
        variant_name_deserialize = #variant_name_deserialize,
        content = #content,
        decoder = <#inner_type as ::jalava::ElmJson>::decoder_type(),
    )};

    Coder { encoder, decoder }
}

/// #[derive(Deserialize, Serialize)]
/// #[serde(tag = "t", content = "c")]
/// enum Adjacent {
///     Tuple(i32, i32),
/// }
/// "{\"t\":\"Tuple\",\"c\":[0,0]}"
fn enum_variant_tuple_adjacent(
    tag: &str,
    content: &str,
    variant_name: &str,
    variant_name_serialize: &str,
    variant_name_deserialize: &str,
    tuple_types: &[Type],
) -> Coder {
    let idx: Vec<usize> = (0..tuple_types.len()).collect();

    let encoder = quote! { format!("\
    {variant_name} {params} ->
            Json.Encode.object [ ( \"{tag}\", Json.Encode.string \"{variant_name_serialize}\"), ( \"{content}\", Json.Encode.list identity [ {encoders} ] ) ]",
        variant_name = #variant_name,
        params = (
            &[
                #(::std::format!("t{}", #idx)
                ),*
            ]
        ).join(" "),
        tag = #tag,
        content = #content,
        variant_name_serialize = #variant_name_serialize,
        encoders = (
            &[
                #(::std::format!("{} t{}", <#tuple_types as ::jalava::ElmJson>::encoder_type(), #idx)
                ),*
            ]
        ).join(", "),
    )};

    let decoder = quote! {::std::format!("\
\"{variant_name_deserialize}\" ->
                        Json.Decode.field \"{content}\" (Json.Decode.succeed {variant_name} {decoders})",
        variant_name = #variant_name,
        variant_name_deserialize = #variant_name_deserialize,
        content = #content,
        decoders = (
            &[
                #(::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.index {} ({}) |> Json.Decode.map x)",
                    #idx,
                    <#tuple_types as ::jalava::ElmJson>::decoder_type())
                ),*
            ]
        ).join(" "),
    )};

    Coder { encoder, decoder }
}

/// #[derive(Deserialize, Serialize)]
/// #[serde(tag = "t", content = "c")]
/// enum Adjacent {
///     Struct { a: i32 },
/// }
/// "{\"t\":\"Struct\",\"c\":{\"a\":0}}"
fn enum_variant_struct_adjacent(
    tag: &str,
    content: &str,
    variant_name: &str,
    variant_name_serialize: &str,
    variant_name_deserialize: &str,
    fields: &[StructField],
) -> (Coder, TokenStream2) {
    let (field_names, tys): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|field| (field.name_elm(), &field.ty))
        .unzip();
    let constructor = constructor(variant_name, &field_names);

    let field_names_serialize = fields.iter().map(|field| field.name_serialize());
    let encoder = quote! { format!("\
    {variant_name} {{ {fields} }} ->
            Json.Encode.object [ ( \"{tag}\", Json.Encode.string \"{variant_name_serialize}\"), ( \"{content}\", Json.Encode.object [ {encoders} ] ) ]",
        variant_name = #variant_name,
        fields = (
            &[
                #(::std::format!("{}", #field_names)),*
            ]
        ).join(", "),
        tag = #tag,
        content = #content,
        variant_name_serialize = #variant_name_serialize,
        encoders = (
            &[
                #(::std::format!("( \"{}\", {} {} )",
                    #field_names_serialize,
                    <#tys as ::jalava::ElmJson>::encoder_type(),
                    #field_names,
                )),*
            ]
        ).join(", "),
    )};

    let mut field_decoders = vec![];
    for field in fields {
        let ty = &field.ty;
        let field_name_deserialize = field.name_deserialize();
        let alias_decoders = field
            .aliases
            .iter()
            .map(|a| quote! {::std::format!(", Json.Decode.field \"{}\" ({})", #a, <#ty as ::jalava::ElmJson>::decoder_type())})
            .collect::<Vec<_>>();
        field_decoders.push(quote!{::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.oneOf [Json.Decode.field \"{field_name_deserialize}\" ({decoder}){alias_decoders}]))",
                field_name_deserialize = #field_name_deserialize,
                decoder = <#ty as ::jalava::ElmJson>::decoder_type(),
                alias_decoders = (&[#(#alias_decoders),*] as &[::std::string::String]).join("")
        )});
    }
    let decoder = quote! {::std::format!("\
\"{variant_name_deserialize}\" ->
                        Json.Decode.field \"{content}\" (Json.Decode.succeed construct{variant_name} {field_decoders})",
        variant_name = #variant_name,
        variant_name_deserialize = #variant_name_deserialize,
        content = #content,
        field_decoders = (
            &[
                #(#field_decoders),*
            ]
        ).join("\n        "),
    )};

    (Coder { encoder, decoder }, constructor)
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
fn enum_variant_unit_untagged(variant_name: &str) -> Coder {
    let encoder = quote! {::std::format!("\
{variant_name} ->
            Json.Encode.null",
        variant_name = #variant_name
    )};

    let decoder = quote! {::std::format!("\
    Json.Decode.null {variant_name}",
        variant_name = #variant_name
    )};

    Coder { encoder, decoder }
}

/// #[derive(Deserialize, Serialize)]
/// #[serde(untagged)]
/// enum Untagged {
///     Newtype(i32),
/// }
/// "0"
fn enum_variant_newtype_untagged(variant_name: &str, inner: &Type) -> Coder {
    let encoder = quote! {::std::format!("\
{variant_name} inner ->
            {encoder} inner",
        variant_name = #variant_name,
        encoder = <#inner as ::jalava::ElmJson>::encoder_type(),
    )};

    let decoder = quote! {::std::format!("\
    Json.Decode.map {variant_name} ({decoder})",
        variant_name = #variant_name,
        decoder = <#inner as ::jalava::ElmJson>::decoder_type(),
    )};

    Coder { encoder, decoder }
}

/// #[derive(Deserialize, Serialize)]
/// #[serde(untagged)]
/// enum Untagged {
///     Tuple(i32, i32),
/// }
/// "[0,0]"
fn enum_variant_tuple_untagged(variant_name: &str, tuple_types: &[Type]) -> Coder {
    let idx: Vec<usize> = (0..tuple_types.len()).collect();
    let encoder = quote! {::std::format!("\
{variant_name} {params} -> 
            Json.Encode.list identity
            [ {encoders}
            ]",
        variant_name = #variant_name,
        params = (
            &[
                #(::std::format!("t{idx}",
                    idx = #idx
                )),*
            ]
        ).join(" "),
        encoders = (
            &[
                #(::std::format!("({}) t{}",
                    <#tuple_types as ::jalava::ElmJson>::encoder_type(),
                    #idx
                )),*
            ]
        ).join("\n            , "),
    )};

    let idx: Vec<usize> = (0..tuple_types.len()).collect();
    let decoder = quote! {::std::format!("\
    Json.Decode.succeed {enum_variant} {decoders}",
        enum_variant = #variant_name,
        decoders = (
            &[
                #(::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.index {} ({}) |> Json.Decode.map x)",
                    #idx,
                    <#tuple_types as ::jalava::ElmJson>::decoder_type()
                )),*
            ]
        ).join(" ")
    )};

    Coder { encoder, decoder }
}

/// #[derive(Deserialize, Serialize)]
/// #[serde(untagged)]
/// enum Untagged {
///     Struct { a: i32 },
/// }
/// "{\"a\":0}"
fn enum_variant_struct_untagged(
    variant_name: &str,
    fields: &[StructField],
) -> (Coder, TokenStream2) {
    let (field_names, tys): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|field| (field.name_elm(), &field.ty))
        .unzip();
    let field_names_deserialize = fields.iter().map(|field| field.name_deserialize());
    let field_names_serialize = fields.iter().map(|field| field.name_serialize());
    let constructor = constructor(variant_name, &field_names);

    let encoder = quote! {::std::format!("\
{variant_name} {{ {fields} }} ->
            Json.Encode.object [ {encoders} ]",
        variant_name = #variant_name,
        fields =  (
            &[
                #(::std::format!("{}", #field_names,
                )),*
            ]
        ).join(", "),
        encoders = (
            &[
                #(::std::format!("( \"{}\", {} {} )",
                    #field_names_serialize,
                    <#tys as ::jalava::ElmJson>::encoder_type(),
                    #field_names,
                )),*
            ]
        ).join(", ")
    )};

    let decoder = quote! {::std::format!("\
    Json.Decode.succeed construct{variant_name} {decoders}",
            variant_name = #variant_name,
            decoders = (
                &[
                    #(::std::format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.field \"{}\" ({})))",
                        #field_names_deserialize,
                        <#tys as ::jalava::ElmJson>::decoder_type()
                    )),*
                ]
            ).join(" "),
    )};

    (Coder { encoder, decoder }, constructor)
}

// #######
// helpers
// #######

fn constructor(variant_name: &str, field_names: &[String]) -> TokenStream2 {
    quote! {::std::format!("\
construct{enum_variant} {fields} =
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
