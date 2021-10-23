use super::{EnumKind, EnumVariant, Intermediate, StructField, TypeInfo};
use heck::CamelCase;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, punctuated::Punctuated, DeriveInput, Type};

pub fn derive_elm(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let intermediate = super::derive_input_to_intermediate(derive_input);
    let token_stream = intermediate_to_token_stream(intermediate);
    TokenStream::from(token_stream)
}

fn intermediate_to_token_stream(
    Intermediate {
        ident,
        generics,
        type_info: kind,
    }: Intermediate,
) -> TokenStream2 {
    let elm_type = ident.to_string().to_camel_case();

    let type_definition = match kind {
        TypeInfo::Unit => unit(&elm_type),
        TypeInfo::Newtype(ty) => newtype(&elm_type, &ty),
        TypeInfo::Tuple(tys) => tuple(&elm_type, &tys),
        TypeInfo::Struct(fields) => struct_type(&elm_type, fields),
        TypeInfo::Enum(variants) => enum_type(&elm_type, variants),
    };

    // prepare a list of generics without any bounds
    let mut without_bounds = generics.clone();
    for param in without_bounds.type_params_mut() {
        param.bounds = Punctuated::default();
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

fn newtype(elm_type: &str, ty: &Type) -> TokenStream2 {
    quote! {format!("\
type {elm_type}
    = {elm_type} ({inner_type})
",
        elm_type = #elm_type,
        inner_type = <#ty>::elm_type(),
    )}
}

fn tuple(elm_type: &str, ts: &[Type]) -> TokenStream2 {
    quote! {format!("\
type {elm_type}
    = {elm_type} {types}
",
        elm_type = #elm_type,
        types = (&[#(format!("({})", <#ts>::elm_type())),*] as &[String]).join(" "),
    )}
}

fn struct_type(elm_type: &str, fields: Vec<StructField>) -> TokenStream2 {
    let ids = fields.iter().map(|field| field.name());
    let tys = fields.iter().map(|field| &field.ty);
    quote! {format!("\
type alias {elm_type} =
    {{ {fields}
    }}
", 
        elm_type = #elm_type,
        fields = (&[#(format!("{} : {}", #ids, <#tys>::elm_type())),*] as &[String]).join("\n    , "),
    )}
}

fn enum_type(elm_type: &str, variants: Vec<EnumVariant>) -> TokenStream2 {
    let mut enum_fields: Vec<TokenStream2> = vec![];
    for variant in variants {
        let id = variant.ident.to_string().to_camel_case();
        match variant.variant {
            EnumKind::Unit => {
                enum_fields.push(quote! {#id});
            }
            EnumKind::Newtype(ty) => {
                enum_fields.push(quote! {format!("{} ({})", #id, <#ty>::elm_type())});
            }
            EnumKind::Tuple(tys) => enum_fields.push(
                    quote! {format!("{} {}", #id, (&[#(format!("({})", <#tys>::elm_type())),*] as &[String]).join(" "))},
                ),
            EnumKind::Struct(fields) => {
                let ids = fields.iter().map(|field| field.name());
                let tys = fields.iter().map(|field| &field.ty);
                enum_fields.push(quote! {format!("{} {{ {} }}", #id, (&[#(format!("{} : {}", #ids, <#tys>::elm_type())),*] as &[String]).join(", "))});
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
