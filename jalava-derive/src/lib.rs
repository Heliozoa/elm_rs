mod elm;
mod elm_form;

use proc_macro::TokenStream;

#[proc_macro_derive(Elm)]
pub fn derive_elm(input: TokenStream) -> TokenStream {
    elm::derive_elm(input)
}

#[proc_macro_derive(ElmForm)]
pub fn derive_elm_form(input: TokenStream) -> TokenStream {
    elm_form::derive_elm_form(input)
}
