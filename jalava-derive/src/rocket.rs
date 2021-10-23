use crate::{EnumKind, Intermediate, StructField, TypeInfo};
use heck::{CamelCase, MixedCase};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

pub fn derive_elm_form(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let intermediate = super::derive_input_to_intermediate(derive_input);
    let token_stream = intermediate_to_form(intermediate);
    TokenStream::from(token_stream)
}

pub fn derive_elm_form_parts(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let intermediate = super::derive_input_to_intermediate(derive_input);
    let token_stream = intermediate_to_fields(intermediate);
    TokenStream::from(token_stream)
}

fn intermediate_to_form(
    Intermediate {
        ident,
        generics,
        type_info: kind,
    }: Intermediate,
) -> TokenStream2 {
    let elm_type = ident.to_string().to_camel_case();
    let form_parts = make_form_parts(&ident, &kind);
    let fields = if let TypeInfo::Struct(fields) = kind {
        fields
    } else {
        panic!("only structs are supported")
    };
    let prepare_form = make_prepare_form(&elm_type, &fields);

    quote! {
        impl #generics jalava::ElmForm for #ident #generics {
            fn prepare_form() -> String {
                #prepare_form
            }
        }

        impl #generics jalava::ElmFormParts for #ident #generics {
            #form_parts
        }
    }
}

fn intermediate_to_fields(
    Intermediate {
        ident,
        generics,
        type_info: kind,
    }: Intermediate,
) -> TokenStream2 {
    let form_parts = make_form_parts(&ident, &kind);

    quote! {
        impl #generics jalava::ElmFormParts for #ident #generics {
            #form_parts
        }
    }
}

fn make_prepare_form(form_type_name: &str, fields: &[StructField]) -> TokenStream2 {
    let field_names = fields.iter().map(|field| &field.ident);
    let field_types = fields.iter().map(|field| &field.ty);

    quote! {
        use jalava::ElmFormParts;
        let form_fields =  [#(<#field_types>::form_parts(stringify!(#field_names))),*];
        format!(
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
                fn form_parts_inner(field: &str, path: &str, recursion: u32) -> String {
                    format!("{}",
                        [#(format!("{}", <#tys>::form_parts_inner(
                            &format!("{}.{}", field, stringify!(#ids)),
                            &format!("{}.{}", path, stringify!(#ids)),
                            recursion + 1
                        ))),*
                        ].join("\n            , "))
                }
            }
        }
        TypeInfo::Enum(fields) => {
            let names = fields
                .iter()
                .map(|variant| match variant.variant {
                    EnumKind::Unit => &variant.ident,
                    _ => panic!("only unit variants are supported"),
                })
                .collect::<Vec<_>>();

            let id = id.to_string();
            let to_string = format!("{}ToString", id.to_mixed_case());
            let to_string_definition = quote! {format!(
                "\
{0} : {1} -> String
{0} enum =
    case enum of
        {2}
",
                #to_string,
                #id,
                [#(format!("{0} -> \"{0}\"", stringify!(#names))),*].join("\n\n        ")
            )};
            quote! {
                fn form_parts_inner(field: &str, path: &str, _recursion: u32) -> String {
                    format!("[ Http.stringPart \"{}\" ({} {}) ]", field, #to_string, path)
                }

                fn to_string() -> Option<String> {
                    Some(#to_string.to_string())
                }

                fn to_string_definition() -> Option<String> {
                    Some(#to_string_definition.to_string())
                }
            }
        }
        _ => {
            panic!("only structs and enums are supported")
        }
    }
}
