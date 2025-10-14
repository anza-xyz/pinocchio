use pinocchio::program_error::ProgramError;
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput};

/// Defines a custom error macro to create custom errors in pinocchio framework
#[proc_macro_derive(ErrorCode, attributes(msg))]
pub fn error_code(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let variants = if let Data::Enum(data_enum) = &input.data {
        &data_enum.variants
    } else {
        return syn::Error::new_spanned(name, "ErrorCode can only be derived for enums")
            .to_compile_error()
            .into();
    };

    let match_arms = variants.iter().map(|variant| {
        let variant_name = &variant.ident;
        let msg = variant
            .attrs
            .iter()
            .find(|attr| attr.path.is_ident("msg"))
            .and_then(|attr| attr.parse_meta().ok())
            .and_then(|meta| {
                if let syn::Meta::NameValue(nv) = meta {
                    if let syn::Lit::Str(lit) = nv.lit {
                        return Some(lit);
                    }
                }
                None
            })
            .unwrap_or_else(|| {
                panic!(
                    "Variant `{}` must have a #[msg(\"...\")] attribute",
                    variant_name
                )
            });
        quote! {
            Self::#variant_name => #msg,
        }
    });

    let expanded = quote! {
        impl #name {
            pub fn message(&self) -> &str {
                match self {
                    #( #match_arms )*
                }
            }
        }

        impl From<#name> for ProgramError {
            fn from(e: #name) -> Self {
                ProgramError::Custom(e as u32)
            }
        }
    };

    TokenStream::from(expanded)
}
