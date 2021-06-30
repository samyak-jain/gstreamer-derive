use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{spanned::Spanned, Attribute, DataEnum, Lit};

use crate::utils::convert_ident_case;

fn parse_token_stream(attribute: &Attribute) -> Result<Lit, TokenStream> {
    let parsed_attribute = match attribute.parse_meta() {
        Ok(meta) => meta,
        Err(err) => return Err(err.to_compile_error()),
    };

    let name_value_attribute = match parsed_attribute {
        syn::Meta::NameValue(name_value_meta) => name_value_meta,
        _ => {
            let token_span = attribute.span();
            return Err(quote_spanned! { token_span =>
                compile_error!("Expected a value to be assigned to name");
            });
        }
    };

    return Ok(name_value_attribute.lit);
}

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
pub fn create_elements(data_enum: &DataEnum) -> Result<(TokenStream, Vec<Ident>), TokenStream> {
    // We initially create an empty [`TokenStream`](proc_macro2::TokenStream)
    // In every iteration, we will append the token generated from the attribute
    let mut element_creation = quote!();
    let variants = &data_enum.variants;
    let mut ident_list: Vec<Ident> = Vec::with_capacity(variants.len());

    'variant_loop: for variant in variants {
        let original_ident = &variant.ident;
        let ident = convert_ident_case(original_ident);
        ident_list.push(ident.clone());

        let attribute_list = &variant.attrs;

        for attribute in attribute_list {
            if !attribute.path.is_ident("name") {
                continue 'variant_loop;
            }

            if attribute.path.is_ident("count") {
                ident_list.pop();

                let literal = parse_token_stream(attribute)?;
                let count = match literal {
                    syn::Lit::Int(count) => match count.base10_parse::<u16>() {
                        Ok(parsed_count) => parsed_count,
                        Err(err) => return Err(err.to_compile_error()),
                    },
                    _ => {
                        let name_token = literal.span();
                        return Err(quote_spanned! { name_token =>
                            compile_error!("name can only be assigned a string literal");
                        });
                    }
                };

                for index in 0..count {
                    let ident_dup = format_ident!("{}_{}", ident, index);
                    ident_list.push(ident_dup);

                    element_creation.extend(quote! {
                        let #ident_dup = gstreamer::ElementFactory::make(
                            &stringify!(#original_ident).to_lowercase(), Some(#name)).unwrap();
                    });
                }
            }

            let literal = parse_token_stream(attribute)?;
            let name = match literal {
                syn::Lit::Str(name) => name.value(),
                _ => {
                    let name_token = literal.span();
                    return Err(quote_spanned! { name_token =>
                        compile_error!("name can only be assigned a string literal");
                    });
                }
            };

            element_creation.extend(quote! {
                let #ident = gstreamer::ElementFactory::make(
                    &stringify!(#original_ident).to_lowercase(), Some(#name)).unwrap();
            });

            continue 'variant_loop;
        }

        element_creation.extend(quote! {
                let #ident = gstreamer::ElementFactory::make(
                    &stringify!(#original_ident).to_lowercase(), Some(stringify!(#ident))).unwrap();
        });
    }

    Ok((element_creation, ident_list))
}
