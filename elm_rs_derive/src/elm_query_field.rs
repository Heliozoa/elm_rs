//! Derive macro for ElmQuery.

use crate::{EnumVariantKind, Intermediate, TypeInfo};
use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

pub fn derive(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as DeriveInput);
    let intermediate = match Intermediate::parse(derive_input) {
        Ok(intermediate) => intermediate,
        Err(err) => return TokenStream::from(err.to_compile_error()),
    };
    let token_stream = match intermediate_to_token_stream(intermediate) {
        Ok(token_stream) => token_stream,
        Err(err) => return TokenStream::from(err.to_compile_error()),
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
    let query_field_encoder_name = format!("queryFieldEncoder{elm_type}");
    let ts = match type_info {
        TypeInfo::Enum { variants, .. } => {
            let mut branches = Vec::new();
            for variant in variants {
                if let EnumVariantKind::Unit = variant.variant {
                    let elm_name = variant.name_elm();
                    let name_encode = variant.name_encode(&container_attributes);
                    branches.push(format!("{elm_name} -> \"{name_encode}\""));
                } else {
                    return Err(syn::Error::new(
                        variant.span,
                        "only unit variants are allowed",
                    ));
                }
            }

            quote! {::std::format!("\
{function_name} : {elm_type} -> String
{function_name} var =
    case var of
        {branches}
",
                function_name = #query_field_encoder_name,
                elm_type = #elm_type,
                branches = (
                    &[
                        #(#branches),*
                    ]
                ).join("\n        ")
            )}
        }
        _ => return Err(syn::Error::new(ident.span(), "only enums are allowed")),
    };

    for p in generics.type_params_mut() {
        p.bounds.push(syn::parse_str("::elm_rs::Elm").unwrap());
        p.bounds
            .push(syn::parse_str("::elm_rs::ElmQueryField").unwrap());
    }

    let res = quote! {
        impl #generics ::elm_rs::ElmQueryField for #ident #generics_without_bounds {
            fn query_field_type() -> &'static str {
                "Url.Builder.string"
            }

            fn query_field_encoder_name() -> &'static str {
                #query_field_encoder_name
            }

            fn query_field_encoder_definition() -> Option<String> {
                Some(#ts)
            }
        }
    };
    Ok(res)
}
