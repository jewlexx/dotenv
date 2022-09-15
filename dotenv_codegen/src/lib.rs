extern crate proc_macro;

use std::env::{self, VarError};
use std::path::Path;

use proc_macro::TokenStream;
use quote::{quote, ToTokens};
use syn::parse::Parser;
use syn::punctuated::Punctuated;
use syn::{Token, VisPublic, Visibility};

/// Load the dotenv file at build time, and set the environment variables at runtime.
#[proc_macro]
pub fn dotenv_build(input: TokenStream) -> TokenStream {
    let path = if input.is_empty() {
        ".env".to_owned()
    } else {
        let args = Punctuated::<syn::Expr, Token![,]>::parse_terminated
            .parse(input)
            .unwrap();

        let mut iter = args.into_iter();

        match iter.next().unwrap() {
            syn::Expr::Assign(expr) => {
                if expr.left.to_token_stream().to_string() == "filename" {
                    let literal =
                        match litrs::StringLit::parse(expr.right.to_token_stream().to_string()) {
                            Ok(v) => v,
                            Err(_) => panic!(),
                        };

                    let value = literal.into_value();

                    value.to_string()
                } else {
                    ".env".to_owned()
                }
            }
            _ => panic!(),
        }
    };

    if let Ok((_, file)) = dotenv::find::Finder::new()
        .filename(Path::new(&path))
        .find()
    {
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

/// Load the dotenv file at build time, and transform all variables into constants within a module.
///
/// Can parse publicity modifier for the module as the first argument.
#[proc_macro]
pub fn dotenv_module(input: TokenStream) -> TokenStream {
    let mut path = ".env".to_owned();
    let mut visibility = Visibility::Public(VisPublic {
        pub_token: Default::default(),
    });

    if !input.is_empty() {
        let args = Punctuated::<syn::Expr, Token![,]>::parse_terminated
            .parse(input)
            .expect("Could not parse arguments");

        for l in args.into_iter() {
            match l {
                syn::Expr::Assign(expr) => {
                    let assigned = expr.left.to_token_stream().to_string();
                    if assigned == "visibility" {
                        let literal =
                            match litrs::StringLit::parse(expr.right.to_token_stream().to_string())
                            {
                                Ok(v) => v,
                                Err(_) => panic!(),
                            };

                        let value: TokenStream = literal.into_value().parse().unwrap();

                        visibility = syn::parse(value).unwrap();
                    } else if assigned == "filename" {
                        let literal =
                            match litrs::StringLit::parse(expr.right.to_token_stream().to_string())
                            {
                                Ok(v) => v,
                                Err(_) => panic!(),
                            };

                        let value = literal.into_value();

                        path = value.to_string();
                    }
                }
                _ => panic!(),
            };
        }
    }

    if let Ok((_, file)) = dotenv::find::Finder::new()
        .filename(Path::new(&path))
        .find()
    {
        let statements = file
            .map(|line| match line {
                Ok((var_name, var_content)) => {
                    let var_name_tokens: proc_macro2::TokenStream = var_name.parse().unwrap();
                    quote! {
                        #visibility const #var_name_tokens: &str = #var_content;
                    }
                }

                Err(e) => {
                    let msg = e.to_string();
                    quote! { compile_error!(#msg) }
                }
            })
            .collect::<Vec<proc_macro2::TokenStream>>();

        quote!(#visibility mod dotenv_vars {
                #(#statements)*
        })
        .into()
    } else {
        panic!("Could not find .env file");
    }
}

/// Find a given variable in the dotenv file at build time
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
