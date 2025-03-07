//! Derive macro for Elm.

use super::{EnumVariant, EnumVariantKind, Intermediate, StructField, TypeInfo};
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
        container_attributes: _,
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
        p.bounds.push(syn::parse_str("::elm_rs::Elm").unwrap());
    }

    quote! {
        impl #generics ::elm_rs::Elm for #ident #generics_without_bounds {
            fn elm_type() -> ::std::string::String {
                ::std::convert::From::from(#elm_type)
            }

            fn elm_definition() -> ::std::option::Option<::std::string::String> {
                ::std::option::Option::Some(#type_definition)
            }
        }
    }
}

fn unit(elm_type: &str) -> TokenStream2 {
    quote! {::std::format!("\
type {elm_type}
    = {elm_type}
",
        elm_type = #elm_type,
    )}
}

fn newtype(elm_type: &str, ty: &Type) -> TokenStream2 {
    quote! {::std::format!("\
type {elm_type}
    = {elm_type} ({inner_type})
",
        elm_type = #elm_type,
        inner_type = <#ty as ::elm_rs::Elm>::elm_type(),
    )}
}

fn tuple(elm_type: &str, ts: &[Type]) -> TokenStream2 {
    quote! {::std::format!("\
type {elm_type}
    = {elm_type} {types}
",
        elm_type = #elm_type,
        types =
            (
                &[
                    #(::std::format!("({})", <#ts as ::elm_rs::Elm>::elm_type())),*
                ]
            ).join(" "),
    )}
}

fn struct_type(elm_type: &str, fields: Vec<StructField>) -> TokenStream2 {
    let ids = fields.iter().map(|field| field.name_elm());
    let tys = fields.iter().map(|field| &field.ty);
    quote! {::std::format!("\
type alias {elm_type} =
    {{ {fields}
    }}
", 
        elm_type = #elm_type,
        fields =
            (
                &[
                    #(::std::format!("{} : {}", #ids, <#tys as ::elm_rs::Elm>::elm_type())),*
                ]
            ).join("\n    , "),
    )}
}

fn enum_type(elm_type: &str, enum_variants: Vec<EnumVariant>) -> TokenStream2 {
    let mut enum_fields: Vec<TokenStream2> = vec![];
    for enum_variant in enum_variants {
        let variant_elm_name = enum_variant.name_elm();
        match &enum_variant.variant {
            EnumVariantKind::Unit => {
                let field = quote! {
                    #variant_elm_name
                };
                enum_fields.push(field);
            }
            EnumVariantKind::Newtype(ty) => {
                let field = quote! {
                    ::std::format!("{} ({})", #variant_elm_name, <#ty as ::elm_rs::Elm>::elm_type())
                };
                enum_fields.push(field);
            }
            EnumVariantKind::Tuple(tys) => {
                let field = quote! {
                    ::std::format!("{name} {types}",
                        name = #variant_elm_name,
                        types =
                            (
                                &[
                                    #(::std::format!("({})", <#tys as ::elm_rs::Elm>::elm_type())),*
                                ] as &[::std::string::String]
                            ).join(" "))
                };
                enum_fields.push(field);
            }
            EnumVariantKind::Struct(fields) => {
                let ids = fields.iter().map(|field| field.name_elm());
                let tys = fields.iter().map(|field| &field.ty);
                let field = quote! {
                    ::std::format!("{name} {{ {fields} }}",
                    name = #variant_elm_name,
                    fields =
                        (
                            &[
                                #(::std::format!("{} : {}", #ids, <#tys as ::elm_rs::Elm>::elm_type())),*
                            ] as &[::std::string::String]
                        ).join(", "))
                };
                enum_fields.push(field);
            }
        }
    }
    quote! {::std::format!("\
type {elm_type}
    = {enum_fields}
", 
        elm_type = #elm_type,
        enum_fields =
            (
                &[
                    #(::std::format!("{}", #enum_fields)),*
                ] as &[::std::string::String]
            ).join("\n    | "),
    )}
}
