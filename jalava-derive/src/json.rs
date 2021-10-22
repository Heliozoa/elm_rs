use super::{Attributes, EnumVariant, Intermediate, TypeKind};
use heck::{CamelCase, MixedCase};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, DeriveInput, Ident, Type};

pub fn derive_elm_json(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let intermediate = super::derive_input_to_intermediate(derive_input);
    let token_stream = intermediate_to_token_stream(intermediate);
    TokenStream::from(token_stream)
}

fn intermediate_to_token_stream(
    Intermediate {
        attributes,
        ident,
        generics,
        kind,
    }: Intermediate,
) -> TokenStream2 {
    let elm_type = ident.to_string().to_camel_case();

    let decoder_type = format!("{}Decoder", elm_type.to_mixed_case());
    let encoder_type = format!("{}Encoder", elm_type.to_mixed_case());

    let (decoder_definition, encoder_definition) = match kind {
        TypeKind::Unit => unit(&elm_type, &decoder_type, &encoder_type),
        TypeKind::Newtype(ty) => newtype(&elm_type, &decoder_type, &encoder_type, &ty),
        TypeKind::Tuple(ts) => tuple(&elm_type, &decoder_type, &encoder_type, &ts),
        TypeKind::Struct(mut fs) => {
            if attributes.serde_transparent && fs.len() == 1 {
                newtype(
                    &elm_type,
                    &decoder_type,
                    &encoder_type,
                    &fs.pop().unwrap().1,
                )
            } else {
                struct_type(&elm_type, &decoder_type, &encoder_type, fs, &attributes)
            }
        }
        TypeKind::Enum(vs) => enum_type(&elm_type, &decoder_type, &encoder_type, vs, &attributes),
    };

    // prepare a list of generics without any bounds
    let mut without_bounds = generics.clone();
    for p in without_bounds.type_params_mut() {
        p.bounds = Punctuated::default();
    }

    quote! {
        impl #generics jalava::ElmJson for #ident #without_bounds {
            fn decoder_type() -> String {
                #decoder_type.to_string()
            }

            fn decoder_definition() -> Option<String> {
                Some(#decoder_definition)
            }

            fn encoder_type() -> String {
                #encoder_type.to_string()
            }

            fn encoder_definition() -> Option<String> {
                Some(#encoder_definition)
            }
        }
    }
}

fn unit(elm_type: &str, decoder_type: &str, encoder_type: &str) -> (TokenStream2, TokenStream2) {
    let dd = quote! {format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    Json.Decode.null {elm_type}
",
        elm_type = #elm_type,
        decoder_type = #decoder_type,
    )};
    let ed = quote! {format!("\
{encoder_type} : {elm_type} -> Json.Encode.Value
{encoder_type} _ =
    Json.Encode.null
",
        elm_type = #elm_type,
        encoder_type = #encoder_type,
    )};
    (dd, ed)
}

fn newtype(
    elm_type: &str,
    decoder_type: &str,
    encoder_type: &str,
    ty: &Type,
) -> (TokenStream2, TokenStream2) {
    let dd = quote! {format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    Json.Decode.map {elm_type} ({inner_decoder})
",
        elm_type = #elm_type,
        decoder_type = #decoder_type,
        inner_decoder = <#ty>::decoder_type(),
    )};
    let ed = quote! {format!("\
{encoder_type} : {elm_type} -> Json.Encode.Value
{encoder_type} ({elm_type} inner) =
    ({inner_encoder}) inner
",
        elm_type = #elm_type,
        encoder_type = #encoder_type,
        inner_encoder = <#ty>::encoder_type(),
    )};
    (dd, ed)
}

fn tuple(
    elm_type: &str,
    decoder_type: &str,
    encoder_type: &str,
    ts: &[Type],
) -> (TokenStream2, TokenStream2) {
    let idx: Vec<usize> = ts.iter().enumerate().map(|(i, _)| i).collect();
    let dd = quote! {format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    Json.Decode.succeed {elm_type}
        {decoders}
",
        elm_type = #elm_type,
        decoder_type = #decoder_type,
        decoders = (&[#(format!("|> Json.Decode.andThen (\\x -> Json.Decode.index {} ({}) |> Json.Decode.map x)", #idx, <#ts>::decoder_type())),*] as &[String]).join("\n        ")
    )};
    let ed = quote! {format!("\
{encoder_type} : {elm_type} -> Json.Encode.Value
{encoder_type} ({elm_type} {params}) =
    Json.Encode.list identity
        [ {encoders}
        ]
",
        elm_type = #elm_type,
        encoder_type = #encoder_type,
        params = (&[#(format!("t{}", #idx)),*] as &[String]).join(" "),
        encoders = (&[#(format!("({}) t{}", <#ts>::encoder_type(), #idx)),*] as &[String]).join("\n        , "),
    )};
    (dd, ed)
}

fn struct_type(
    elm_type: &str,
    decoder_type: &str,
    encoder_type: &str,
    fs: Vec<(Ident, Type)>,
    attributes: &Attributes,
) -> (TokenStream2, TokenStream2) {
    let (ids, ts): (Vec<_>, Vec<_>) = fs
        .into_iter()
        .map(|(i, t)| (super::convert_case(&i, attributes), t))
        .unzip();
    let dd = quote! {format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    Json.Decode.succeed {elm_type}
        {fields}
",
        elm_type = #elm_type,
        decoder_type = #decoder_type,
        fields = (&[#(format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.field \"{}\" ({})))", #ids, <#ts>::decoder_type())),*] as &[String]).join("\n        "),
    )};
    let ed = quote! {format!("\
{encoder_type} : {elm_type} -> Json.Encode.Value
{encoder_type} struct =
    Json.Encode.object
        [ {fields}
        ]
",
        elm_type = #elm_type,
        encoder_type = #encoder_type,
        fields = (&[#(format!("( \"{0}\", ({1}) struct.{0} )", #ids, <#ts>::encoder_type())),*] as &[String]).join("\n        , "),
    )};
    (dd, ed)
}

fn enum_type(
    elm_type: &str,
    decoder_type: &str,
    encoder_type: &str,
    vs: Vec<(Ident, EnumVariant)>,
    attributes: &Attributes,
) -> (TokenStream2, TokenStream2) {
    let mut enum_fields: Vec<TokenStream2> = vec![];
    let mut constructors: Vec<TokenStream2> = vec![];
    let mut decoders: Vec<TokenStream2> = vec![];
    let mut encoders: Vec<TokenStream2> = vec![];
    for (id, ev) in vs {
        let id_s = id.to_string().to_camel_case();
        match ev {
            EnumVariant::Unit => enum_unit(&mut enum_fields, &mut decoders, &mut encoders, &id_s),
            EnumVariant::Newtype(ty) => {
                enum_newtype(&mut enum_fields, &mut decoders, &mut encoders, &id_s, &ty);
            }
            EnumVariant::Tuple(tuple_types) => {
                enum_tuple(
                    &mut enum_fields,
                    &mut decoders,
                    &mut encoders,
                    &id_s,
                    &tuple_types,
                );
            }
            EnumVariant::Struct(fs) => enum_struct(
                &mut enum_fields,
                &mut decoders,
                &mut encoders,
                &mut constructors,
                &id_s,
                attributes,
                &fs,
            ),
        }
    }
    let decoder = if constructors.is_empty() {
        quote! {format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    Json.Decode.oneOf
        [ {decoders}        ]
",
            elm_type = #elm_type,
            decoder_type = #decoder_type,
            decoders = (&[#(format!("{}", #decoders)),*] as &[String]).join("        , "),
        )}
    } else {
        quote! {format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    let
        {constructors}    in
    Json.Decode.oneOf
        [ {decoders}        ]
",
            constructors = (&[#(format!("{}", #constructors)),*] as &[String]).join("        "),
            elm_type = #elm_type,
            decoder_type = #decoder_type,
            decoders = (&[#(format!("{}", #decoders)),*] as &[String]).join("        , "),
        )}
    };
    let encoder = quote! {format!("\
{encoder_type} : {elm_type} -> Json.Encode.Value
{encoder_type} enum =
    case enum of
        {encoders}",
        elm_type = #elm_type,
        encoder_type = #encoder_type,
        encoders = (&[#(format!("{}", #encoders)),*] as &[String]).join("\n        ")
    )};
    (decoder, encoder)
}

fn enum_unit(
    enum_fields: &mut Vec<TokenStream2>,
    decoders: &mut Vec<TokenStream2>,
    encoders: &mut Vec<TokenStream2>,
    id: &str,
) {
    enum_fields.push(quote! {#id});
    decoders.push(quote! {format!("\
Json.Decode.andThen
            (\\x ->
                case x of
                    \"{enum_variant}\" ->
                        Json.Decode.succeed {enum_variant}

                    _ ->
                        Json.Decode.fail \"invalid enum variant\"
            )
            Json.Decode.string
", 
        enum_variant = #id,
    )});
    encoders.push(quote! {format!("\
        {enum_variant} ->
            Json.Encode.string \"{enum_variant}\"
",
        enum_variant = #id,
    )});
}

fn enum_newtype(
    enum_fields: &mut Vec<TokenStream2>,
    decoders: &mut Vec<TokenStream2>,
    encoders: &mut Vec<TokenStream2>,
    id: &str,
    ty: &Type,
) {
    enum_fields.push(quote! {format!("{} ({})", #id, <#ty>::elm_type())});
    decoders.push(quote! {format!("\
Json.Decode.map {enum_variant} (Json.Decode.field \"{enum_variant}\" ({decoder}))
",
        enum_variant = #id,
        decoder = <#ty>::decoder_type(),
    )});
    encoders.push(quote! {format!("\
            {enum_variant} inner ->
                Json.Encode.object [ ( \"{enum_variant}\", {encoder} inner ) ]
",
        enum_variant = #id,
        encoder = <#ty>::encoder_type(),
    )});
}

fn enum_tuple(
    enum_fields: &mut Vec<TokenStream2>,
    decoders: &mut Vec<TokenStream2>,
    encoders: &mut Vec<TokenStream2>,
    id: &str,
    tuple_types: &[Type],
) {
    enum_fields.push(
        quote! {format!("{} {}", #id, (&[#(format!("({})", <#tuple_types>::elm_type())),*] as &[String]).join(" "))},
    );
    let idx: Vec<usize> = (0..tuple_types.len()).collect();
    decoders.push(quote! {format!("\
Json.Decode.field \"{enum_variant}\"
            (Json.Decode.succeed {enum_variant}
                {decoders}
            )
",
        enum_variant = #id,
        decoders = (&[#(format!("|> Json.Decode.andThen (\\x -> Json.Decode.index {} ({}) |> Json.Decode.map x)", #idx, <#tuple_types>::decoder_type())),*] as &[String]).join("\n                ")
    )});
    encoders.push(quote! {format!("\
        {enum_variant} {params} ->
            Json.Encode.object [ ( \"{enum_variant}\", Json.Encode.list identity [ {encoders} ] ) ]
",
        enum_variant = #id,
        params = (&[#(format!("t{}", #idx)),*] as &[String]).join(" "),
        encoders = (&[#(format!("{} t{}", <#tuple_types>::encoder_type(), #idx)),*] as &[String]).join(", "),
    )});
}

fn enum_struct(
    enum_fields: &mut Vec<TokenStream2>,
    decoders: &mut Vec<TokenStream2>,
    encoders: &mut Vec<TokenStream2>,
    constructors: &mut Vec<TokenStream2>,
    id: &str,
    attributes: &Attributes,
    fs: &[(Ident, Type)],
) {
    let (ids, tys): (Vec<_>, Vec<_>) = fs
        .iter()
        .map(|(i, t)| (super::convert_case(i, attributes), t))
        .unzip();
    enum_fields.push(quote! {format!("{} {{ {} }}", #id, (&[#(format!("{} : {}", #ids, <#tys>::elm_type())),*] as &[String]).join(", "))});
    constructors.push(quote! {format!("\
        construct{enum_variant} {fields} =
            {enum_variant} {{ {setters} }}
",
        enum_variant = #id,
        fields = (&[#(format!("{}", #ids)),*] as &[String]).join(" "),
        setters = (&[#(format!("{0} = {0}", #ids)),*] as &[String]).join(", "),
    )});
    decoders.push(quote! {format!("\
Json.Decode.field \"{enum_variant}\"
            (Json.Decode.succeed construct{enum_variant}
                {decoders}
            )
",
                    enum_variant = #id,
                    decoders = (&[#(format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.field \"{}\" {}))", #ids, <#tys>::decoder_type())),*] as &[String]).join("\n                "),
                )});
    encoders.push(quote! {format!("\
        {enum_variant} {{ {fields} }} ->
            Json.Encode.object [ ( \"{enum_variant}\", Json.Encode.object [ {encoders} ] ) ]
",
                    enum_variant = #id,
                    fields = (&[#(format!("{}", #ids)),*] as &[String]).join(", "),
                    encoders = (&[#(format!("( \"{0}\", {1} {0} )", #ids, <#tys>::encoder_type())),*] as &[String]).join(", ")
                )});
}
