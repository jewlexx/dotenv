extern crate proc_macro;

use std::env::{self, VarError};

use proc_macro::TokenStream;
use quote::quote;
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::Token;

#[proc_macro]
pub fn dotenv_build(_: TokenStream) -> TokenStream {
    if let Ok((_, file)) = dotenv::find::Finder::new().find() {
        let statements = file
            .map(|line| match line {
                Ok((var_name, var_content)) => {
                    let stmt = quote! {
                        std::env::set_var(#var_name, #var_content);
                    };

                    stmt
                }
                Err(e) => {
                    let msg = e.to_string();
                    quote! { compile_error!(#msg) }
                }
            })
            .collect::<Vec<proc_macro2::TokenStream>>();

        quote!(#(#statements)*).into()
    } else {
        panic!("Could not find .env file");
    }
}

#[proc_macro]
pub fn dotenv_module(_: TokenStream) -> TokenStream {
    if let Ok((_, file)) = dotenv::find::Finder::new().find() {
        let statements = file
            .map(|line| match line {
                Ok((var_name, var_content)) => {
                    let var_name_tokens: proc_macro2::TokenStream = var_name.parse().unwrap();
                    quote! {
                        const #var_name_tokens: &str = #var_content;
                    }
                }

                Err(e) => {
                    let msg = e.to_string();
                    quote! { compile_error!(#msg) }
                }
            })
            .collect::<Vec<proc_macro2::TokenStream>>();

        quote!(mod dotenv_vars {
                #(#statements)*
        })
        .into()
    } else {
        panic!("Could not find .env file");
    }
}

#[proc_macro]
pub fn dotenv(input: TokenStream) -> TokenStream {
    if let Err(err) = dotenv::dotenv() {
        panic!("Error loading .env file: {}", err);
    }

    // Either everything was fine, or we didn't find an .env file (which we ignore)
    expand_env(input)
}

fn expand_env(input_raw: TokenStream) -> TokenStream {
    let args = <Punctuated<syn::LitStr, Token![,]>>::parse_terminated
        .parse(input_raw)
        .expect("expected macro to be called with a comma-separated list of string literals");

    let mut iter = args.iter();

    let var_name = match iter.next() {
        Some(s) => s.value(),
        None => panic!("expected 1 or 2 arguments, found none"),
    };

    let err_msg = match iter.next() {
        Some(lit) => lit.value(),
        None => format!("environment variable `{}` not defined", var_name),
    };

    if iter.next().is_some() {
        panic!("expected 1 or 2 arguments, found 3 or more");
    }

    match env::var(var_name) {
        Ok(val) => quote!(#val).into(),
        Err(VarError::NotPresent) | Err(VarError::NotUnicode(_)) => panic!("{}", err_msg),
    }
}
