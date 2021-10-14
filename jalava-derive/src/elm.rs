use super::{Attributes, EnumVariant, Intermediate, TypeKind};
use heck::CamelCase;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident, Type};

pub fn derive_elm(input: TokenStream) -> TokenStream {
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

    let type_definition = match kind {
        TypeKind::Unit => unit(&elm_type),
        TypeKind::Newtype(ty) => newtype(&elm_type, ty),
        TypeKind::Tuple(ts) => tuple(&elm_type, ts),
        TypeKind::Struct(mut fs) => {
            if attributes.serde_transparent && fs.len() == 1 {
                newtype(&elm_type, fs.pop().unwrap().1)
            } else {
                struct_type(&elm_type, fs, &attributes)
            }
        }
        TypeKind::Enum(vs) => enum_type(&elm_type, vs, &attributes),
    };

    // prepare a list of generics without any bounds
    let mut without_bounds = generics.clone();
    for p in without_bounds.type_params_mut() {
        p.bounds = Default::default();
    }

    quote! {
        impl #generics jalava::Elm for #ident #without_bounds {
            fn elm_type() -> String {
                #elm_type.to_string()
            }

            fn elm_definition() -> Option<String> {
                Some(#type_definition)
            }
        }
    }
}

fn unit(elm_type: &str) -> TokenStream2 {
    quote! {format!("\
type {elm_type}
    = {elm_type}
",
        elm_type = #elm_type,
    )}
}

fn newtype(elm_type: &str, ty: Type) -> TokenStream2 {
    quote! {format!("\
type {elm_type}
    = {elm_type} ({inner_type})
",
        elm_type = #elm_type,
        inner_type = <#ty>::elm_type(),
    )}
}

fn tuple(elm_type: &str, ts: Vec<Type>) -> TokenStream2 {
    quote! {format!("\
type {elm_type}
    = {elm_type} {types}
",
        elm_type = #elm_type,
        types = (&[#(format!("({})", <#ts>::elm_type())),*] as &[String]).join(" "),
    )}
}

fn struct_type(elm_type: &str, fs: Vec<(Ident, Type)>, attributes: &Attributes) -> TokenStream2 {
    let (ids, ts): (Vec<_>, Vec<_>) = fs
        .into_iter()
        .map(|(i, t)| (super::convert_case(i, attributes), t))
        .unzip();
    quote! {format!("\
type alias {elm_type} =
    {{ {fields}
    }}
", 
        elm_type = #elm_type,
        fields = (&[#(format!("{} : {}", #ids, <#ts>::elm_type())),*] as &[String]).join("\n    , "),
    )}
}

fn enum_type(
    elm_type: &str,
    vs: Vec<(Ident, EnumVariant)>,
    attributes: &Attributes,
) -> TokenStream2 {
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
                    quote! {format!("{} {}", #id_s, (&[#(format!("({})", <#tuple_types>::elm_type())),*] as &[String]).join(" "))},
                );
                let idx: Vec<usize> = (0..tuple_types.len()).collect();
                decoders.push(quote! {format!("\
Json.Decode.field \"{enum_variant}\"
            (Json.Decode.succeed {enum_variant}
                {decoders}
            )
",
                    enum_variant = #id_s,
                    decoders = (&[#(format!("|> Json.Decode.andThen (\\x -> Json.Decode.index {} ({}) |> Json.Decode.map x)", #idx, <#tuple_types>::decoder_type())),*] as &[String]).join("\n                ")
                )});
                encoders.push(quote! {format!("\
        {enum_variant} {params} ->
            Json.Encode.object [ ( \"{enum_variant}\", Json.Encode.list identity [ {encoders} ] ) ]
",
                    enum_variant = #id_s,
                    params = (&[#(format!("t{}", #idx)),*] as &[String]).join(" "),
                    encoders = (&[#(format!("{} t{}", <#tuple_types>::encoder_type(), #idx)),*] as &[String]).join(", "),
                )});
            }
            EnumVariant::Struct(fs) => {
                let (ids, tys): (Vec<_>, Vec<_>) = fs
                    .into_iter()
                    .map(|(i, t)| (super::convert_case(i, attributes), t))
                    .unzip();
                enum_fields.push(quote! {format!("{} {{ {} }}", #id_s, (&[#(format!("{} : {}", #ids, <#tys>::elm_type())),*] as &[String]).join(", "))});
                constructors.push(quote! {format!("\
        construct{enum_variant} {fields} =
            {enum_variant} {{ {setters} }}
",
                    enum_variant = #id_s,
                    fields = (&[#(format!("{}", #ids)),*] as &[String]).join(" "),
                    setters = (&[#(format!("{0} = {0}", #ids)),*] as &[String]).join(", "),
                )});
                decoders.push(quote! {format!("\
Json.Decode.field \"{enum_variant}\"
            (Json.Decode.succeed construct{enum_variant}
                {decoders}
            )
",
                    enum_variant = #id_s,
                    decoders = (&[#(format!("|> Json.Decode.andThen (\\x -> Json.Decode.map x (Json.Decode.field \"{}\" {}))", #ids, <#tys>::decoder_type())),*] as &[String]).join("\n                "),
                )});
                encoders.push(quote! {format!("\
        {enum_variant} {{ {fields} }} ->
            Json.Encode.object [ ( \"{enum_variant}\", Json.Encode.object [ {encoders} ] ) ]
",
                    enum_variant = #id_s,
                    fields = (&[#(format!("{}", #ids)),*] as &[String]).join(", "),
                    encoders = (&[#(format!("( \"{0}\", {1} {0} )", #ids, <#tys>::encoder_type())),*] as &[String]).join(", ")
                )});
            }
        }
    }
    quote! {format!("\
type {elm_type}
    = {enum_fields}
", 
        elm_type = #elm_type,
        enum_fields = (&[#(format!("{}", #enum_fields)),*] as &[String]).join("\n    | "),
    )}
}
