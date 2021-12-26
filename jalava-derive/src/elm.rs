//! Derive macro for Elm.

use super::{EnumVariant, EnumVariantKind, Intermediate, StructField, TypeInfo};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Type};

pub fn derive_elm(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let intermediate = match super::derive_input_to_intermediate(derive_input) {
        Ok(intermediate) => intermediate,
        Err(err) => return err.to_compile_error().into(),
    };
    let token_stream = intermediate_to_token_stream(intermediate);
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
) -> TokenStream2 {
    let type_definition = match type_info {
        TypeInfo::Unit => unit(&elm_type),
        TypeInfo::Newtype(ty) => newtype(&elm_type, &ty),
        TypeInfo::Tuple(tys) => tuple(&elm_type, &tys),
        TypeInfo::Struct(fields) => struct_type(&elm_type, fields),
        TypeInfo::Enum { variants, .. } => enum_type(&elm_type, variants),
    };

    for p in generics.type_params_mut() {
        p.bounds.push(syn::parse_str("::jalava::Elm").unwrap());
    }

    quote! {
        impl #generics ::jalava::Elm for #ident #generics_without_bounds {
            fn elm_type() -> ::std::string::String {
                ::std::string::String::from(#elm_type)
            }

            fn elm_definition() -> ::std::option::Option<::std::string::String> {
                ::std::option::Option::Some(#type_definition)
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
        inner_type = <#ty as ::jalava::Elm>::elm_type(),
    )}
}

fn tuple(elm_type: &str, ts: &[Type]) -> TokenStream2 {
    quote! {format!("\
type {elm_type}
    = {elm_type} {types}
",
        elm_type = #elm_type,
        types =
            (
                &[
                    #(format!("({})", <#ts as ::jalava::Elm>::elm_type())),*
                ] as &[String]
            ).join(" "),
    )}
}

fn struct_type(elm_type: &str, fields: Vec<StructField>) -> TokenStream2 {
    let ids = fields.iter().map(|field| field.name_elm());
    let tys = fields.iter().map(|field| &field.ty);
    quote! {format!("\
type alias {elm_type} =
    {{ {fields}
    }}
", 
        elm_type = #elm_type,
        fields =
            (
                &[
                    #(format!("{} : {}", #ids, <#tys as ::jalava::Elm>::elm_type())),*
                ] as &[::std::string::String]
            ).join("\n    , "),
    )}
}

fn enum_type(elm_type: &str, enum_variants: Vec<EnumVariant>) -> TokenStream2 {
    let mut enum_fields: Vec<TokenStream2> = vec![];
    for enum_variant in enum_variants {
        let variant_elm_name = enum_variant.name_elm();
        match &enum_variant.variant {
            EnumVariantKind::Unit { .. } => {
                let field = quote! {
                    #variant_elm_name
                };
                enum_fields.push(field);
            }
            EnumVariantKind::Newtype(ty) => {
                let field = quote! {
                    format!("{} ({})", #variant_elm_name, <#ty as ::jalava::Elm>::elm_type())
                };
                enum_fields.push(field);
            }
            EnumVariantKind::Tuple(tys) => {
                let field = quote! {
                    format!("{name} {types}",
                        name = #variant_elm_name,
                        types =
                            (
                                &[
                                    #(format!("({})", <#tys as ::jalava::Elm>::elm_type())),*
                                ] as &[::std::string::String]
                            ).join(" "))
                };
                enum_fields.push(field);
            }
            EnumVariantKind::Struct(fields) => {
                let ids = fields.iter().map(|field| field.name_elm());
                let tys = fields.iter().map(|field| &field.ty);
                let field = quote! {
                    format!("{name} {{ {fields} }}",
                    name = #variant_elm_name,
                    fields =
                        (
                            &[
                                #(format!("{} : {}", #ids, <#tys as ::jalava::Elm>::elm_type())),*
                            ] as &[::std::string::String]
                        ).join(", "))
                };
                enum_fields.push(field);
            }
        }
    }
    quote! {format!("\
type {elm_type}
    = {enum_fields}
", 
        elm_type = #elm_type,
        enum_fields =
            (
                &[
                    #(format!("{}", #enum_fields)),*
                ] as &[::std::string::String]
            ).join("\n    | "),
    )}
}
