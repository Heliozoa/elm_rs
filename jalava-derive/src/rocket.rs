//! Derive macros for Rocket forms.

use crate::{EnumVariantKind, Intermediate, StructField, TypeInfo};
use heck::ToLowerCamelCase;
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

pub fn derive_elm_form(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let intermediate = match super::derive_input_to_intermediate(derive_input) {
        Ok(intermediate) => intermediate,
        Err(err) => return err.to_compile_error().into(),
    };
    let token_stream = match intermediate_to_form(intermediate) {
        Ok(token_stream) => token_stream,
        Err(err) => return err.to_compile_error().into(),
    };
    TokenStream::from(token_stream)
}

pub fn derive_elm_form_parts(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let intermediate = match super::derive_input_to_intermediate(derive_input) {
        Ok(intermediate) => intermediate,
        Err(err) => return err.to_compile_error().into(),
    };
    let token_stream = intermediate_to_fields(intermediate);
    TokenStream::from(token_stream)
}

fn intermediate_to_form(
    Intermediate {
        ident,
        elm_type,
        generics,
        generics_without_bounds,
        type_info,
    }: Intermediate,
) -> Result<TokenStream2, syn::Error> {
    let form_parts = make_form_parts(&ident, &type_info);
    let fields = if let TypeInfo::Struct(fields) = type_info {
        fields
    } else {
        return Err(syn::Error::new(ident.span(), "only structs are supported"));
    };
    let prepare_form = make_prepare_form(&elm_type, &fields);

    let block = quote! {
        impl #generics ::jalava::ElmForm for #ident #generics_without_bounds {
            fn prepare_form() -> ::std::string::String {
                #prepare_form
            }
        }

        impl #generics ::jalava::ElmFormParts for #ident #generics_without_bounds {
            #form_parts
        }
    };
    Ok(block)
}

fn intermediate_to_fields(
    Intermediate {
        ident,
        generics,
        type_info: kind,
        ..
    }: Intermediate,
) -> TokenStream2 {
    let form_parts = make_form_parts(&ident, &kind);

    quote! {
        impl #generics ::jalava::ElmFormParts for #ident #generics {
            #form_parts
        }
    }
}

fn make_prepare_form(form_type_name: &str, fields: &[StructField]) -> TokenStream2 {
    let field_names = fields.iter().map(|field| &field.ident);
    let field_types = fields.iter().map(|field| &field.ty);

    quote! {
        let form_fields =  [#(<#field_types as ::jalava::ElmFormParts>::form_parts(::std::stringify!(#field_names))),*];
        ::std::format!(
                "prepare{0} : {0} -> Http.Body
prepare{0} form =
    Http.multipartBody <|
        List.concat
            [ {1}
            ]
",
            #form_type_name,
            form_fields
                .join("\n            , ")
        )
    }
}

fn make_form_parts(id: &Ident, kind: &TypeInfo) -> TokenStream2 {
    match kind {
        TypeInfo::Struct(fields) => {
            let ids = fields.iter().map(|field| &field.ident);
            let tys = fields.iter().map(|field| &field.ty);
            quote! {
                fn form_parts_inner(field: &::std::primitive::str, path: &::std::primitive::str, recursion: ::std::primitive::u32) -> ::std::string::String {
                    ::std::format!("{}",
                        [#(::std::format!("{}", <#tys as ::jalava::ElmFormParts>::form_parts_inner(
                            &::std::format!("{}.{}", field, ::std::stringify!(#ids)),
                            &::std::format!("{}.{}", path, ::std::stringify!(#ids)),
                            recursion + 1
                        ))),*
                        ].join("\n            , "))
                }
            }
        }
        TypeInfo::Enum { variants, .. } => {
            let names = variants
                .iter()
                .map(|variant| match variant.variant {
                    EnumVariantKind::Unit { .. } => &variant.ident,
                    _ => panic!("only unit variants are supported"),
                })
                .collect::<Vec<_>>();

            let id = id.to_string();
            let to_string = format!("{}ToString", id.to_lower_camel_case());
            let to_string_definition = quote! {::std::format!(
                "\
{0} : {1} -> String
{0} enum =
    case enum of
        {2}
",
                #to_string,
                #id,
                [#(::std::format!("{0} -> \"{0}\"", ::std::stringify!(#names))),*].join("\n\n        ")
            )};
            quote! {
                fn form_parts_inner(field: &::std::primitive::str, path: &::std::primitive::str, _recursion: ::std::primitive::u32) -> ::std::string::String {
                    ::std::format!("[ Http.stringPart \"{}\" ({} {}) ]", field, #to_string, path)
                }

                fn to_string() -> ::std::option::Option<::std::string::String> {
                    ::std::option::Option::Some(::std::string::ToString::to_string(&#to_string))
                }

                fn to_string_definition() -> ::std::option::Option<::std::string::String> {
                    ::std::option::Option::Some(::std::string::ToString::to_string(&#to_string_definition))
                }
            }
        }
        _ => {
            panic!("only structs and enums are supported")
        }
    }
}
