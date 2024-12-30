use proc_macro::{self, TokenStream};
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

use bevy_macro_utils::{get_struct_fields, BevyManifest};

use advice::Advice;

mod advice;

#[proc_macro_derive(Advice, attributes(advice, help))]
pub fn derive_advice(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let cmd = match Advice::from_derive_input(input) {
        Ok(cmd) => cmd.gen(),
        Err(err) => return err.to_compile_error().into(),
    };
    // panic!("{:#}", cmd.to_token_stream());
    quote!(#cmd).into()
}

pub(crate) fn bevy_error_path() -> syn::Path {
    BevyManifest::default().get_path("bevy_error")
}
