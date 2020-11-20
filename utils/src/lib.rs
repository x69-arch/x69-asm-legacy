extern crate proc_macro;
use proc_macro::TokenStream;
use quote::quote;
use std::stringify;
use syn::{parse_macro_input, Data::Enum, DeriveInput};

#[proc_macro_derive(ToFromString)]
pub fn to_from_string(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);
    let name = &ast.ident;
    // eprintln!("{:#?}", ast);
    
    let variants = match ast.data {
        Enum(e) => {
            let mut out = Vec::new();
            out.extend(e.variants.iter().map(|e| e.ident.clone()));
            out
        },
        _ => panic!("#[derive(ToFromString)] is only implemented for enums!")
    };
    
    let generated = quote! {
        impl #name {
            #[inline(always)]
            pub fn to_str(&self) -> &'static str {
                match self {
                    #(Self::#variants => stringify!(#variants)),*
                }
            }
            #[inline(always)]
            pub fn from_str(string: &str) -> Option<Self> {
                match string {
                    #(stringify!(#variants) => Some(Self::#variants),)*
                    _ => None
                }
            }
        }
    };
    
    generated.into()
}
