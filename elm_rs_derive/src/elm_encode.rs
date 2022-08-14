//! Derive macro for ElmEncode.

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
    let encoder_type = format!("{}Encoder", elm_type.to_lower_camel_case());

    let encoder = match type_info {
        TypeInfo::Unit => struct_unit(&elm_type, &encoder_type),
        TypeInfo::Newtype(ty) => struct_newtype(&elm_type, &encoder_type, &ty),
        TypeInfo::Tuple(tys) => struct_tuple(&elm_type, &encoder_type, &tys),
        TypeInfo::Struct(fields) => {
            struct_named(&elm_type, &encoder_type, &fields, &container_attributes)
        }
        TypeInfo::Enum {
            variants,
            #[cfg(feature = "serde")]
            representation,
        } => {
            #[cfg(feature = "serde")]
            let representation = match representation {
                EnumRepresentation::External => {
                    enum_external(&elm_type, &encoder_type, variants, &container_attributes)
                }
                EnumRepresentation::Internal { tag } => enum_internal(
                    &elm_type,
                    &encoder_type,
                    variants,
                    &tag,
                    &container_attributes,
                )?,
                EnumRepresentation::Adjacent { tag, content } => enum_adjacent(
                    &elm_type,
                    &encoder_type,
                    variants,
                    &tag,
                    &content,
                    &container_attributes,
                )?,
                EnumRepresentation::Untagged => {
                    enum_untagged(&elm_type, &encoder_type, variants, &container_attributes)?
                }
            };
            #[cfg(not(feature = "serde"))]
            let representation =
                enum_external(&elm_type, &encoder_type, variants, &container_attributes);
            representation
        }
    };

    for p in generics.type_params_mut() {
        p.bounds.push(syn::parse_str("::elm_rs::Elm").unwrap());
        p.bounds
            .push(syn::parse_str("::elm_rs::ElmEncode").unwrap());
    }

    let res = quote! {
        impl #generics ::elm_rs::ElmEncode for #ident #generics_without_bounds {
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

/// #[derive(Deserialize, Serialize)]
/// struct Unit;
/// "null"
fn struct_unit(elm_type: &str, encoder_type: &str) -> TokenStream2 {
    quote! {::std::format!("\
{encoder_type} : {elm_type} -> Json.Encode.Value
{encoder_type} _ =
    Json.Encode.null
",
        elm_type = #elm_type,
        encoder_type = #encoder_type,
    )}
}

/// #[derive(Deserialize, Serialize)]
/// struct Newtype(i32);
/// "0"
fn struct_newtype(elm_type: &str, encoder_type: &str, ty: &Type) -> TokenStream2 {
    quote! {::std::format!("\
{encoder_type} : {elm_type} -> Json.Encode.Value
{encoder_type} ({elm_type} inner) =
    ({inner_encoder}) inner
",
        elm_type = #elm_type,
        encoder_type = #encoder_type,
        inner_encoder = <#ty>::encoder_type(),
    )}
}

/// #[derive(Deserialize, Serialize)]
/// struct Tuple(i32, i32);
/// "[0,0]"
fn struct_tuple(elm_type: &str, encoder_type: &str, inner_types: &[Type]) -> TokenStream2 {
    let indices: Vec<usize> = inner_types.iter().enumerate().map(|(i, _)| i).collect();

    quote! {::std::format!("\
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
    )}
}

/// #[derive(Deserialize, Serialize)]
/// struct Struct {
///     a: i32,
/// };
/// "{\"a\":0}"
fn struct_named(
    elm_type: &str,
    encoder_type: &str,
    fields: &[StructField],
    container_attributes: &ContainerAttributes,
) -> TokenStream2 {
    let mut field_encoders = vec![];
    for field in fields {
        let field_name = field.name_elm();
        let field_name_encode = field.name_encode(container_attributes);
        let ty = &field.ty;
        field_encoders.push(
            quote! {::std::format!("( \"{field_name_encode}\", ({encoder}) struct.{field_name} )",
            field_name_encode = #field_name_encode,
            encoder = <#ty as ::elm_rs::ElmEncode>::encoder_type(),
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

    encoder
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
    variants: Vec<EnumVariant>,
    container_attributes: &ContainerAttributes,
) -> TokenStream2 {
    let mut encoders = vec![];
    for variant in variants {
        #[cfg(feature = "serde")]
        if variant.serde_attributes.skip {
            continue;
        }

        let elm_name = variant.name_elm();
        let elm_name_encode = variant.name_encode(container_attributes);

        let encoder = match &variant.variant {
            EnumVariantKind::Unit => enum_variant_unit_external(&elm_name, &elm_name_encode),
            EnumVariantKind::Newtype(inner) => {
                enum_variant_newtype_external(&elm_name, &elm_name_encode, inner)
            }
            EnumVariantKind::Tuple(types) => {
                enum_variant_tuple_external(&elm_name, &elm_name_encode, types)
            }
            EnumVariantKind::Struct(fields) => enum_variant_struct_external(
                &elm_name,
                &elm_name_encode,
                fields,
                container_attributes,
            ),
        };
        encoders.push(encoder);
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
    encoder
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
    encoder_name: &str,
    variants: Vec<EnumVariant>,
    tag: &str,
    container_attributes: &ContainerAttributes,
) -> Result<TokenStream2, syn::Error> {
    let mut encoders = vec![];
    for variant in variants {
        #[cfg(feature = "serde")]
        if variant.serde_attributes.skip {
            continue;
        }

        let elm_name = variant.name_elm();
        let elm_name_encode = variant.name_encode(container_attributes);

        let encoder = match &variant.variant {
            EnumVariantKind::Unit => {
                enum_variant_unit_internal_or_adjacent(tag, &elm_name, &elm_name_encode)
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
            EnumVariantKind::Struct(fields) => enum_variant_struct_internal(
                tag,
                &elm_name,
                &elm_name_encode,
                fields,
                container_attributes,
            ),
        };
        encoders.push(encoder);
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
    Ok(encoder)
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
    encoder_name: &str,
    variants: Vec<EnumVariant>,
    tag: &str,
    content: &str,
    container_attributes: &ContainerAttributes,
) -> Result<TokenStream2, syn::Error> {
    let mut encoders = vec![];
    for variant in variants {
        #[cfg(feature = "serde")]
        if variant.serde_attributes.skip {
            continue;
        }

        let elm_name = variant.name_elm();
        let elm_name_encode = variant.name_encode(container_attributes);

        let encoder = match &variant.variant {
            EnumVariantKind::Unit => {
                enum_variant_unit_internal_or_adjacent(tag, &elm_name, &elm_name_encode)
            }
            EnumVariantKind::Newtype(inner) => {
                enum_variant_newtype_adjacent(tag, content, &elm_name, &elm_name_encode, inner)
            }
            EnumVariantKind::Tuple(types) => {
                enum_variant_tuple_adjacent(tag, content, &elm_name, &elm_name_encode, types)
            }
            EnumVariantKind::Struct(fields) => enum_variant_struct_adjacent(
                tag,
                content,
                &elm_name,
                &elm_name_encode,
                fields,
                container_attributes,
            ),
        };
        encoders.push(encoder);
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
    Ok(encoder)
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
    encoder_name: &str,
    variants: Vec<EnumVariant>,
    container_attributes: &ContainerAttributes,
) -> Result<TokenStream2, syn::Error> {
    let mut encoders = vec![];
    for variant in variants {
        #[cfg(feature = "serde")]
        if variant.serde_attributes.skip {
            continue;
        }

        let elm_name = variant.name_elm();
        let encoder = match &variant.variant {
            EnumVariantKind::Unit => enum_variant_unit_untagged(&elm_name),
            EnumVariantKind::Newtype(inner) => enum_variant_newtype_untagged(&elm_name, inner),
            EnumVariantKind::Tuple(types) => enum_variant_tuple_untagged(&elm_name, types),
            EnumVariantKind::Struct(fields) => {
                enum_variant_struct_untagged(&elm_name, fields, container_attributes)
            }
        };
        encoders.push(encoder);
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
    Ok(encoder)
}

// =================
// external variants
// =================

/// #[derive(Deserialize, Serialize)]
/// enum External {
///     Unit,
/// }
/// "\"Unit\""
fn enum_variant_unit_external(variant_name: &str, variant_name_encode: &str) -> TokenStream2 {
    quote! {::std::format!("\
    {variant_name} ->
            Json.Encode.string \"{variant_name_encode}\"",
        variant_name = #variant_name,
        variant_name_encode = #variant_name_encode,
    )}
}

/// #[derive(Deserialize, Serialize)]
/// enum External {
///     Newtype(i32),
/// }
/// "{\"Newtype\":0}"
fn enum_variant_newtype_external(
    variant_name: &str,
    variant_name_encode: &str,
    inner_type: &TokenStream2,
) -> TokenStream2 {
    quote! { format!("\
{variant_name} inner ->
            Json.Encode.object [ ( \"{variant_name_encode}\", {encoder} inner ) ]",
        variant_name = #variant_name,
        variant_name_encode = #variant_name_encode,
        encoder = <#inner_type as ::elm_rs::ElmEncode>::encoder_type(),
    )}
}

/// #[derive(Deserialize, Serialize)]
/// enum External {
///     Tuple(i32, i32),
/// }
/// "{\"Tuple\":[0,0]}"
fn enum_variant_tuple_external(
    variant_name: &str,
    variant_name_encode: &str,
    tuple_types: &[TokenStream2],
) -> TokenStream2 {
    let idx: Vec<usize> = (0..tuple_types.len()).collect();
    quote! {::std::format!("\
{variant_name} {fields} ->
            Json.Encode.object [ ( \"{variant_name_encode}\", Json.Encode.list identity [ {encoders} ] ) ]",
        variant_name = #variant_name,
        fields = (
            &[
                #(::std::format!("t{}", #idx)
                ),*
            ]
        ).join(" "),
        variant_name_encode = #variant_name_encode,
        encoders = (
            &[
                #(::std::format!("{} t{}",
                    <#tuple_types as ::elm_rs::ElmEncode>::encoder_type(),
                    #idx
                )),*
            ]
        ).join(", "),
    )}
}

/// #[derive(Deserialize, Serialize)]
/// enum External {
///     Struct { a: i32 },
/// }
/// "{\"Struct\":{\"a\":0}}"
fn enum_variant_struct_external(
    variant_name: &str,
    variant_name_encode: &str,
    fields: &[StructField],
    container_attributes: &ContainerAttributes,
) -> TokenStream2 {
    let (field_names, tys): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|field| (field.name_elm(), &field.ty))
        .unzip();
    let field_names_serialize = fields
        .iter()
        .map(|field| field.name_encode(container_attributes));

    quote! {::std::format!("\
{variant_name} {{ {fields} }} ->
            Json.Encode.object [ ( \"{variant_name_encode}\", Json.Encode.object [ {encoders} ] ) ]",
        variant_name = #variant_name,
        variant_name_encode = #variant_name_encode,
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
                    <#tys as ::elm_rs::ElmEncode>::encoder_type(),
                    #field_names,
                )),*
            ]
        ).join(", ")
    )}
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
    tag: &str,
    variant_name: &str,
    variant_name_encode: &str,
) -> TokenStream2 {
    quote! {::std::format!("\
{variant_name} ->
            Json.Encode.object [ ( \"{tag}\", Json.Encode.string \"{variant_name_encode}\" ) ]",
        variant_name = #variant_name,
        tag = #tag,
        variant_name_encode = #variant_name_encode,
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
    tag: &str,
    variant_name: &str,
    variant_name_encode: &str,
    fields: &[StructField],
    container_attributes: &ContainerAttributes,
) -> TokenStream2 {
    let (field_names, tys): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|field| (field.name_elm(), &field.ty))
        .unzip();
    let field_names_serialize = fields
        .iter()
        .map(|field| field.name_encode(container_attributes));

    quote! {::std::format!("\
{variant_name} {{ {fields} }} ->
            Json.Encode.object [ ( \"{tag}\", Json.Encode.string \"{variant_name_encode}\" ), {encoders} ]",
        variant_name = #variant_name,
        fields = (&[
                #(#field_names),*
        ]).join(", "),
        tag = #tag,
        variant_name_encode = #variant_name_encode,
        encoders = (
            &[
                #(::std::format!("( \"{}\", {} {} )",
                    #field_names_serialize,
                    <#tys>::encoder_type(),
                    #field_names,
                )),*
            ]
        ).join(", "),
    )}
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
    tag: &str,
    content: &str,
    variant_name: &str,
    variant_name_encode: &str,
    inner_type: &TokenStream2,
) -> TokenStream2 {
    quote! { format!("\
    {variant_name} inner ->
            Json.Encode.object [ ( \"{tag}\", Json.Encode.string \"{variant_name_encode}\"), ( \"{content}\", {encoder} inner ) ]",
        variant_name = #variant_name,
        tag = #tag,
        content = #content,
        variant_name_encode = #variant_name_encode,
        encoder = <#inner_type as ::elm_rs::ElmEncode>::encoder_type(),
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
    tag: &str,
    content: &str,
    variant_name: &str,
    variant_name_encode: &str,
    tuple_types: &[TokenStream2],
) -> TokenStream2 {
    let idx: Vec<usize> = (0..tuple_types.len()).collect();

    quote! { format!("\
    {variant_name} {params} ->
            Json.Encode.object [ ( \"{tag}\", Json.Encode.string \"{variant_name_encode}\"), ( \"{content}\", Json.Encode.list identity [ {encoders} ] ) ]",
        variant_name = #variant_name,
        params = (
            &[
                #(::std::format!("t{}", #idx)
                ),*
            ]
        ).join(" "),
        tag = #tag,
        content = #content,
        variant_name_encode = #variant_name_encode,
        encoders = (
            &[
                #(::std::format!("{} t{}", <#tuple_types as ::elm_rs::ElmEncode>::encoder_type(), #idx)
                ),*
            ]
        ).join(", "),
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
    tag: &str,
    content: &str,
    variant_name: &str,
    variant_name_encode: &str,
    fields: &[StructField],
    container_attributes: &ContainerAttributes,
) -> TokenStream2 {
    let (field_names, tys): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|field| (field.name_elm(), &field.ty))
        .unzip();
    let field_names_serialize = fields
        .iter()
        .map(|field| field.name_encode(container_attributes));

    quote! { format!("\
    {variant_name} {{ {fields} }} ->
            Json.Encode.object [ ( \"{tag}\", Json.Encode.string \"{variant_name_encode}\"), ( \"{content}\", Json.Encode.object [ {encoders} ] ) ]",
        variant_name = #variant_name,
        fields = (
            &[
                #(::std::format!("{}", #field_names)),*
            ]
        ).join(", "),
        tag = #tag,
        content = #content,
        variant_name_encode = #variant_name_encode,
        encoders = (
            &[
                #(::std::format!("( \"{}\", {} {} )",
                    #field_names_serialize,
                    <#tys as ::elm_rs::ElmEncode>::encoder_type(),
                    #field_names,
                )),*
            ]
        ).join(", "),
    )}
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
{variant_name} ->
            Json.Encode.null",
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
{variant_name} inner ->
            {encoder} inner",
        variant_name = #variant_name,
        encoder = <#inner as ::elm_rs::ElmEncode>::encoder_type(),
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
                    <#tuple_types as ::elm_rs::ElmEncode>::encoder_type(),
                    #idx
                )),*
            ]
        ).join("\n            , "),
    )};

    encoder
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
) -> TokenStream2 {
    let (field_names, tys): (Vec<_>, Vec<_>) = fields
        .iter()
        .map(|field| (field.name_elm(), &field.ty))
        .unzip();
    let field_names_serialize = fields
        .iter()
        .map(|field| field.name_encode(container_attributes));
    quote! {::std::format!("\
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
                    <#tys as ::elm_rs::ElmEncode>::encoder_type(),
                    #field_names,
                )),*
            ]
        ).join(", ")
    )}
}
