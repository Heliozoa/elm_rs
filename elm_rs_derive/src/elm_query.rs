//! Derive macro for ElmQuery.

use crate::{Intermediate, TypeInfo};
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
    let ts = match type_info {
        TypeInfo::Struct(fields) => {
            let mut query_fields = vec![];
            for field in fields {
                let ty = &field.ty;
                let field_name = field.name_elm();
                let field_name_encode = field.name_encode(&container_attributes);
                query_fields.push(quote! {::std::format!("\
                {field_type} \"{field_name_encode}\" ({query_field_encoder} {field_value})",
                    field_type = <#ty as ::elm_rs::ElmQueryField>::query_field_type(),
                    query_field_encoder = <#ty as ::elm_rs::ElmQueryField>::query_field_encoder_name(),
                    field_name_encode = #field_name_encode,
                    field_value = ::std::format!("struct.{field_name}",
                        field_name = #field_name
                    ),
                )});
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
        _ => {
            return Err(syn::Error::new(
                ident.span(),
                "only structs with named fields are allowed",
            ))
        }
    };

    for p in generics.type_params_mut() {
        p.bounds.push(syn::parse_str("::elm_rs::Elm").unwrap());
        p.bounds
            .push(syn::parse_str("::elm_rs::ElmQueryField").unwrap());
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
