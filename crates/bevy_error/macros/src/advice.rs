use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

pub enum Advice {
    Struct {
        generics: syn::Generics,
        ident: syn::Ident,
        fields: syn::Fields,
    },
    Enum {
        ident: syn::Ident,
        generics: syn::Generics,
    },
}

impl Advice {
    pub fn from_derive_input(input: DeriveInput) -> Result<Self, syn::Error> {
        let input_attrs = input
            .attrs
            .iter()
            .filter(|x| x.path().is_ident("advice"))
            .collect::<Vec<&syn::Attribute>>();
        let advice = match input.data {
            syn::Data::Struct(data_struct) => Advice::Struct {
                fields: data_struct.fields,
                ident: input.ident,
                generics: input.generics,
            },
            syn::Data::Enum(syn::DataEnum { variants, .. }) => Advice::Enum {
                ident: input.ident,
                generics: input.generics,
            },
            syn::Data::Union(_) => {
                return Err(syn::Error::new(
                    input.ident.span(),
                    "Can't derive Advice for Unions",
                ))
            }
        };
        Ok(advice)
    }

    pub fn gen(&self) -> TokenStream {
        let error_path = crate::bevy_error_path();

        match self {
            Self::Struct {
                generics,
                ident,
                fields,
            } => {
                let (impl_generics, ty_generics, where_clause) = &generics.split_for_impl();
                quote! {
                    impl #impl_generics #error_path::Advice for #ident #ty_generics #where_clause {}
                }
            }
            Self::Enum { generics, ident } => {
                let (impl_generics, ty_generics, where_clause) = &generics.split_for_impl();
                quote! {
                    impl #impl_generics #error_path::Advice for #ident #ty_generics #where_clause {}
                }
            }
        }
    }
}
