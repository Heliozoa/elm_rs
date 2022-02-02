//! Derive macro for ElmQuery.

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use crate::{EnumVariantKind, Intermediate, TypeInfo};

pub fn derive(input: TokenStream) -> TokenStream {
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
    let ts = match type_info {
        TypeInfo::Struct(fields) => {
            let mut query_fields = vec![];
            for field in fields {
                let ty = &field.ty;
                let field_name = field.name_elm();
                let field_name_serialize = field.name_serialize();
                query_fields.push(
                    quote! {::std::format!("{query_type} \"{field_name_serialize}\" {field_value}",
                        query_type = <#ty as ::elm_rs::ElmQueryField>::field_type(),
                        field_name_serialize = #field_name_serialize,
                        field_value = ::std::format!("struct.{field_name}",
                            field_name = #field_name
                        ),
                    )},
                );
            }
            quote! {::std::format!("\
urlEncode{elm_type} : {elm_type} -> List Url.Builder.QueryParameter
urlEncode{elm_type} struct =
    [ {fields} ]
",
                elm_type = #elm_type,
                fields = (
                    &[
                        #(#query_fields),*
                    ]
                ).join(", ")
            )}
        }
        TypeInfo::Enum { variants, .. } => {
            // enum representation is ignored, probably safe?
            let mut query_variants = vec![];
            for variant in variants {
                if let EnumVariantKind::Struct(fields) = &variant.variant {
                    let elm_name = variant.name_elm();
                    let field_names: Vec<_> = fields.iter().map(|field| field.name_elm()).collect();

                    let mut query_fields = vec![];
                    for field in fields {
                        let ty = &field.ty;
                        let field_name = field.name_elm();
                        let field_name_serialize = field.name_serialize();
                        query_fields.push(
                            quote! {::std::format!("{query_type} \"{field_name_serialize}\" {field_value}",
                                query_type = <#ty as ::elm_rs::ElmQueryField>::field_type(),
                                field_name_serialize = #field_name_serialize,
                                field_value = #field_name
                            )},
                        );
                    }
                    query_variants.push(quote! {::std::format!("\
{variant_name} {{ {fields} }} ->
            [ {query_fields} ]",
                        variant_name = #elm_name,
                        fields = (&[
                            #(#field_names),*
                        ]).join(", "),
                        query_fields = (
                            &[
                                #(#query_fields),*
                            ]
                        ).join(", ")
                    )});
                } else {
                    return Err(syn::Error::new(
                        variant.span,
                        "Only struct variants are allowed",
                    ));
                }
            }
            quote! {::std::format!("\
urlEncode{elm_type} : {elm_type} -> List Url.Builder.QueryParameter
urlEncode{elm_type} enum =
    case enum of
        {variants}
",
                elm_type = #elm_type,
                variants = (
                    &[
                        #(#query_variants),*
                    ]
                ).join("\n        ")
            )}
        }
        _ => {
            return Err(syn::Error::new(
                ident.span(),
                "Only structs and enums are allowed",
            ))
        }
    };

    for p in generics.type_params_mut() {
        p.bounds.push(syn::parse_str("::elm_rs::Elm").unwrap());
        p.bounds.push(syn::parse_str("::elm_rs::ElmJson").unwrap());
    }

    let res = quote! {
        impl #generics ::elm_rs::ElmQuery for #ident #generics_without_bounds {
            fn elm_query() -> ::std::string::String {
                #ts
            }
        }
    };
    Ok(res)
}
