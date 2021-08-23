use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Data, DataStruct, DeriveInput, Field, Fields, FieldsNamed, Ident, Path,
    Type, TypePath,
};

pub fn derive_elm_form(input: TokenStream) -> TokenStream {
    todo!();
    let input = parse_macro_input!(input as DeriveInput);

    let mut fields = vec![];
    if let Data::Struct(DataStruct {
        fields: Fields::Named(FieldsNamed { named, .. }),
        ..
    }) = &input.data
    {
        for field in named {
            if let Field {
                ident: Some(ident),
                ty: Type::Path(TypePath { path, .. }),
                ..
            } = field
            {
                fields.push((ident, path));
            } else {
                panic!("unexpected field")
            }
        }
    } else {
        panic!("unexpected struct")
    };

    let ident = input.ident;
    let form_type_name = ident.to_string();
    let fn_form_type = make_form_type(&form_type_name, &fields);
    let fn_prepare_form = make_prepare_form(&form_type_name, &fields);
    let expanded = quote! {
        impl ElmForm for #ident {
            #fn_form_type

            #fn_prepare_form
        }
    };
    TokenStream::from(expanded)
}

fn make_form_type(form_type_name: &str, fields: &[(&Ident, &Path)]) -> proc_macro2::TokenStream {
    let field_names = fields.iter().map(|v| v.0);
    let field_types = fields.iter().map(|v| v.1);

    quote! {
        fn form_type() -> String {
            use jalava::{ElmFormField, Elm};
            let field_type_names =  [#(format!("{} : {}", stringify!(#field_names), <#field_types>::elm_type())),*];
            format!(
                "type alias {} =
    {{ {}
    }}
",
                #form_type_name,
                field_type_names
                    .join("\n    , ")
            )
        }
    }
}

fn make_prepare_form(form_type_name: &str, fields: &[(&Ident, &Path)]) -> proc_macro2::TokenStream {
    let field_names = fields.iter().map(|v| v.0);
    let field_types = fields.iter().map(|v| v.1);

    quote! {
        fn prepare_form() -> String {
            use jalava::ElmFormField;
            let field_type_names =  [#(<#field_types>::to_form_fields(stringify!(#field_names))),*];
            format!(
                "prepare{0} : {0} -> Http.Body
prepare{0} form =
    Http.multipartBody
        ({1})
",
                #form_type_name,
                field_type_names
                    .join(" ++\n        ")
            )
        }
    }
}
