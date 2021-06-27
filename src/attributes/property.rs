use quote::{quote, quote_spanned};
use syn::{punctuated::Punctuated, DataEnum, Token};

use crate::utils::convert_ident_case;

// The goal of this function is to create a TokenStream output such that we create an output which
// looks like this following:
// let element = gstreamer::ElementFactory::make("element", Some("element_name"));
//
// Where element is the name of the gstreamer element
// and element_name is the name that the user can assign using an attribute.
//
// # Example:
//
// ```
// #[derive(GSteamer)]
// enum GStreamerData {
//   #[name = "input"]
//   uridecodebin
// }
// ```
//
// should generate something like this:
// ```
// let uridecodebin = gstreamer::ElementFactory::make("uridecodebin", Some("input"));
// ```
//
pub fn set_property(data_enum: &DataEnum) -> proc_macro2::TokenStream {
    // We initially create an empty [`TokenStream`](proc_macro2::TokenStream)
    // In every iteration, we will append the token generated from the attribute
    let mut property_setter = quote!();
    let variants = &data_enum.variants;

    for variant in variants {
        let original_ident = &variant.ident;
        let ident = convert_ident_case(original_ident);
        let attribute_list = &variant.attrs;

        for attribute in attribute_list {
            if !attribute.path.is_ident("property") {
                continue;
            }

            let punctuated_parser =
                Punctuated::<syn::ExprAssign, Token![,]>::parse_separated_nonempty;
            let parsed_tokens = match attribute.parse_args_with(punctuated_parser) {
                Ok(tree) => tree,
                Err(err) => {
                    return err.to_compile_error();
                }
            };

            for expression in parsed_tokens {
                let lhs = match *expression.left {
                    syn::Expr::Path(path_expression) => match path_expression.path.get_ident() {
                        Some(ident) => ident.to_string(),
                        None => return quote! {},
                    },
                    _ => return quote! {},
                };

                property_setter.extend(match *expression.right {
                    syn::Expr::Lit(lit_expr) => match lit_expr.lit {
                        syn::Lit::Str(expr) => {
                            let value = expr.value();
                            quote! {
                                #ident.set_property_from_str(#lhs, #value);
                            }
                        }
                        syn::Lit::Char(expr) => {
                            let value = expr.value();
                            quote! {
                                #ident.set_property(#lhs, #value).unwrap();
                            }
                        }
                        syn::Lit::Int(expr) => match expr.base10_parse::<i64>() {
                            Ok(value) => {
                                quote! {
                                    #ident.set_property(#lhs, #value).unwrap();
                                }
                            }
                            Err(_) => {
                                quote! {}
                            }
                        },
                        syn::Lit::Float(expr) => match expr.base10_parse::<f64>() {
                            Ok(value) => {
                                quote! {
                                    #ident.set_property(#lhs, #value);
                                }
                            }
                            Err(_) => {
                                quote! {}
                            }
                        },
                        syn::Lit::Bool(expr) => {
                            let value = expr.value();
                            quote! {
                                #ident.set_property(#lhs, #value).unwrap();
                            }
                        }
                        _ => {
                            quote! {}
                        }
                    },
                    _ => {
                        quote! {}
                    }
                });
            }
        }
    }

    property_setter
}
