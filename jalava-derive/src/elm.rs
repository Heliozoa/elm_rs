use heck::{CamelCase, MixedCase};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, Data, DataEnum, DeriveInput, Fields, Generics, Ident, Type};

pub fn derive_elm(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let intermediate = derive_input_to_intermediate(derive_input);
    let token_stream = intermediate_to_token_stream(intermediate);
    TokenStream::from(token_stream)
}

struct Intermediate {
    ident: Ident,
    generics: Generics,
    kind: TypeKind,
}

enum TypeKind {
    // struct S;
    // null
    Unit,
    // struct S(String);
    // "string"
    Newtype(Type),
    // struct S(String, u32);
    // []
    // ["string", 0]
    Tuple(Vec<Type>),
    // struct S {
    //     s: String,
    // }
    // {}
    // {"s": "string"}
    Struct(Vec<(Ident, Type)>),
    // enum E {
    //     Variant,
    // }
    Enum(Vec<(Ident, EnumVariant)>),
}

enum EnumVariant {
    // Variant,
    // "Variant"
    Unit,
    // Variant(String),
    // {"Variant": "string"}
    Newtype(Type),
    // Variant(String, u32),
    // {"Variant": []}
    // {"Variant": ["string", 0]}
    Tuple(Vec<Type>),
    // Variant {
    //     s: String,
    // }
    // {}
    // {"s": "string"}
    Struct(Vec<(Ident, Type)>),
}

// parses the input to an intermediate representation that's convenient to turn into the end result
fn derive_input_to_intermediate(input: DeriveInput) -> Intermediate {
    if input.generics.lt_token.is_some() {
        // panic!("{:?}", generics)
    }

    let type_kind = match input.data {
        Data::Struct(data_struct) => match data_struct.fields {
            Fields::Unit => TypeKind::Unit,
            Fields::Unnamed(mut unnamed) => {
                if unnamed.unnamed.len() == 1 {
                    TypeKind::Newtype(unnamed.unnamed.pop().unwrap().into_value().ty)
                } else {
                    TypeKind::Tuple(unnamed.unnamed.into_iter().map(|u| u.ty).collect())
                }
            }
            Fields::Named(named) => TypeKind::Struct(
                named
                    .named
                    .into_iter()
                    .map(|f| (f.ident.unwrap(), f.ty))
                    .collect(),
            ),
        },
        Data::Enum(DataEnum { variants, .. }) => {
            if variants.is_empty() {
                panic!("empty enums not supported");
            }
            TypeKind::Enum(
                variants
                    .into_iter()
                    .map(|v| {
                        (
                            v.ident,
                            match v.fields {
                                Fields::Unit => EnumVariant::Unit,
                                Fields::Unnamed(mut unnamed) => {
                                    if unnamed.unnamed.len() == 1 {
                                        EnumVariant::Newtype(
                                            unnamed.unnamed.pop().unwrap().into_value().ty,
                                        )
                                    } else {
                                        EnumVariant::Tuple(
                                            unnamed.unnamed.into_iter().map(|u| u.ty).collect(),
                                        )
                                    }
                                }
                                Fields::Named(named) => EnumVariant::Struct(
                                    named
                                        .named
                                        .into_iter()
                                        .map(|f| (f.ident.unwrap(), f.ty))
                                        .collect(),
                                ),
                            },
                        )
                    })
                    .collect(),
            )
        }
        Data::Union(_) => panic!("unions are not supported"),
    };
    Intermediate {
        ident: input.ident,
        generics: input.generics,
        kind: type_kind,
    }
}

fn intermediate_to_token_stream(
    Intermediate {
        ident,
        generics,
        kind,
    }: Intermediate,
) -> TokenStream2 {
    let elm_type = ident.to_string().to_camel_case();
    let decoder_type = format!("{}Decoder", elm_type.to_mixed_case());
    let encoder_type = format!("{}Encoder", elm_type.to_mixed_case());

    let (type_definition, decoder_definition, encoder_definition) = match kind {
        TypeKind::Unit => unit(&elm_type, &decoder_type, &encoder_type),
        TypeKind::Newtype(ty) => newtype(&elm_type, &decoder_type, &encoder_type, ty),
        TypeKind::Tuple(ts) => tuple(&elm_type, &decoder_type, &encoder_type, ts),
        TypeKind::Struct(fs) => struct_type(&elm_type, &decoder_type, &encoder_type, fs),
        TypeKind::Enum(vs) => enum_type(&elm_type, &decoder_type, &encoder_type, vs),
    };

    let res = quote! {
        impl #generics jalava::Elm for #ident #generics {
            fn elm_type() -> String {
                #elm_type.to_string()
            }

            fn elm_definition() -> Option<String> {
                Some(#type_definition)
            }

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
    };
    res
}

fn unit(
    elm_type: &str,
    decoder_type: &str,
    encoder_type: &str,
) -> (TokenStream2, TokenStream2, TokenStream2) {
    let td = quote! {format!("\
type {elm_type}
    = {elm_type}
",
        elm_type = #elm_type,
    )};
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
    (td, dd, ed)
}

fn newtype(
    elm_type: &str,
    decoder_type: &str,
    encoder_type: &str,
    ty: Type,
) -> (TokenStream2, TokenStream2, TokenStream2) {
    let td = quote! {format!("\
type {elm_type}
    = {elm_type} ({inner_type})
",
        elm_type = #elm_type,
        inner_type = <#ty>::elm_type(),
    )};
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
    (td, dd, ed)
}

fn tuple(
    elm_type: &str,
    decoder_type: &str,
    encoder_type: &str,
    ts: Vec<Type>,
) -> (TokenStream2, TokenStream2, TokenStream2) {
    let idx: Vec<usize> = ts.iter().enumerate().map(|(i, _)| i).collect();
    let td = quote! {format!("\
type {elm_type}
    = {elm_type} {types}
",
        elm_type = #elm_type,
        types = [#(format!("({})", <#ts>::elm_type())),*].join(" "),
    )};
    let dd = quote! {format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    Json.Decode.succeed {elm_type}
        {decoders}
",
        elm_type = #elm_type,
        decoder_type = #decoder_type,
        decoders = [#(format!("|> Json.Decode.andThen (\\x -> Json.Decode.index {} ({}) |> Json.Decode.map x)", #idx, <#ts>::decoder_type())),*].join("\n        ")
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
        params = [#(format!("t{}", #idx)),*].join(" "),
        encoders = [#(format!("({}) t{}", <#ts>::encoder_type(), #idx)),*].join("\n        , "),
    )};
    (td, dd, ed)
}

fn struct_type(
    elm_type: &str,
    decoder_type: &str,
    encoder_type: &str,
    fs: Vec<(Ident, Type)>,
) -> (TokenStream2, TokenStream2, TokenStream2) {
    let (ids, ts): (Vec<_>, Vec<_>) = fs.into_iter().map(|(i, t)| (i.to_string(), t)).unzip();
    let td = quote! {format!("\
type alias {elm_type} =
    {{ {fields}
    }}
", 
        elm_type = #elm_type,
        fields = [#(format!("{} : {}", #ids, <#ts>::elm_type())),*].join("\n    , "),
    )};
    let dd = quote! {format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    Json.Decode.succeed {elm_type}
        {fields}
",
        elm_type = #elm_type,
        decoder_type = #decoder_type,
        fields = [#(format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.field \"{}\" ({})))", #ids, <#ts>::decoder_type())),*].join("\n        "),
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
        fields = [#(format!("( \"{0}\", ({1}) struct.{0} )", #ids, <#ts>::encoder_type())),*].join("\n        , "),
    )};
    (td, dd, ed)
}

fn enum_type(
    elm_type: &str,
    decoder_type: &str,
    encoder_type: &str,
    vs: Vec<(Ident, EnumVariant)>,
) -> (TokenStream2, TokenStream2, TokenStream2) {
    let mut enum_fields: Vec<TokenStream2> = vec![];
    let mut constructors: Vec<TokenStream2> = vec![];
    let mut decoders: Vec<TokenStream2> = vec![];
    let mut encoders: Vec<TokenStream2> = vec![];
    for (id, ev) in vs {
        let id_s = id.to_string().to_camel_case();
        match ev {
            EnumVariant::Unit => {
                enum_fields.push(quote! {#id_s});
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
                    enum_variant = #id_s,
                )});
                encoders.push(quote! {format!("\
        {enum_variant} ->
            Json.Encode.string \"{enum_variant}\"
",
                    enum_variant = #id_s,
                )});
            }
            EnumVariant::Newtype(ty) => {
                enum_fields.push(quote! {format!("{} ({})", #id_s, <#ty>::elm_type())});
                decoders.push(quote! {format!("\
Json.Decode.map {enum_variant} (Json.Decode.field \"{enum_variant}\" ({decoder}))
",
                    enum_variant = #id_s,
                    decoder = <#ty>::decoder_type(),
                )});
                encoders.push(quote! {format!("\
        {enum_variant} inner ->
            Json.Encode.object [ ( \"{enum_variant}\", {encoder} inner ) ]
",
                    enum_variant = #id_s,
                    encoder = <#ty>::encoder_type(),
                )});
            }
            EnumVariant::Tuple(tuple_types) => {
                enum_fields.push(
                    quote! {format!("{} {}", #id_s, [#(format!("({})", <#tuple_types>::elm_type())),*].join(" "))},
                );
                let idx: Vec<usize> = (0..tuple_types.len()).collect();
                decoders.push(quote! {format!("\
Json.Decode.field \"{enum_variant}\"
            (Json.Decode.succeed {enum_variant}
                {decoders}
            )
",
                    enum_variant = #id_s,
                    decoders = [#(format!("|> Json.Decode.andThen (\\x -> Json.Decode.index {} ({}) |> Json.Decode.map x)", #idx, <#tuple_types>::decoder_type())),*].join("\n                ")
                )});
                encoders.push(quote! {format!("\
        {enum_variant} {params} ->
            Json.Encode.object [ ( \"{enum_variant}\", Json.Encode.list identity [ {encoders} ] ) ]
",
                    enum_variant = #id_s,
                    params = [#(format!("t{}", #idx)),*].join(" "),
                    encoders = [#(format!("{} t{}", <#tuple_types>::encoder_type(), #idx)),*].join(", "),
                )});
            }
            EnumVariant::Struct(fs) => {
                let (ids, tys): (Vec<_>, Vec<_>) =
                    fs.into_iter().map(|(i, t)| (i.to_string(), t)).unzip();
                enum_fields.push(quote! {format!("{} {{ {} }}", #id_s, [#(format!("{} : {}", #ids, <#tys>::elm_type())),*].join(", "))});
                constructors.push(quote! {format!("\
        construct{enum_variant} {fields} =
            {enum_variant} {{ {setters} }}
",
                    enum_variant = #id_s,
                    fields = [#(format!("{}", #ids)),*].join(" "),
                    setters = [#(format!("{0} = {0}", #ids)),*].join(", "),
                )});
                decoders.push(quote! {format!("\
Json.Decode.field \"{enum_variant}\"
            (Json.Decode.succeed construct{enum_variant}
                {decoders}
            )
",
                    enum_variant = #id_s,
                    decoders = [#(format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.field \"{}\" {}))", #ids, <#tys>::decoder_type())),*].join("\n                "),
                )});
                encoders.push(quote! {format!("\
        {enum_variant} {{ {fields} }} ->
            Json.Encode.object [ ( \"{enum_variant}\", Json.Encode.object [ {encoders} ] ) ]
",
                    enum_variant = #id_s,
                    fields = [#(format!("{}", #ids)),*].join(", "),
                    encoders = [#(format!("( \"{0}\", {1} {0} )", #ids, <#tys>::encoder_type())),*].join(", ")
                )});
            }
        }
    }
    let td = quote! {format!("\
type {elm_type}
    = {enum_fields}
", 
        elm_type = #elm_type,
        enum_fields = [#(format!("{}", #enum_fields)),*].join("\n    | "),
    )};
    let dd = if constructors.is_empty() {
        quote! {format!("\
{decoder_type} : Json.Decode.Decoder {elm_type}
{decoder_type} =
    Json.Decode.oneOf
        [ {decoders}        ]
",
            elm_type = #elm_type,
            decoder_type = #decoder_type,
            decoders = [#(format!("{}", #decoders)),*].join("        , "),
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
            constructors = [#(format!("{}", #constructors)),*].join("        "),
            elm_type = #elm_type,
            decoder_type = #decoder_type,
            decoders = [#(format!("{}", #decoders)),*].join("        , "),
        )}
    };
    let ed = quote! {format!("\
{encoder_type} : {elm_type} -> Json.Encode.Value
{encoder_type} enum =
    case enum of
        {encoders}",
        elm_type = #elm_type,
        encoder_type = #encoder_type,
        encoders = [#(format!("{}", #encoders)),*].join("\n        ")
    )};
    (td, dd, ed)
}
