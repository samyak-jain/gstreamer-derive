use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, DataEnum};

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
pub fn create_elements(data_enum: &DataEnum) -> proc_macro2::TokenStream {
    // We initially create an empty [`TokenStream`](proc_macro2::TokenStream)
    // In every iteration, we will append the token generated from the attribute
    let mut element_creation = quote!();
    let variants = &data_enum.variants;

    'variant_loop: for variant in variants {
        let original_ident = &variant.ident;
        let ident = convert_ident_case(original_ident);
        let attribute_list = &variant.attrs;

        for attribute in attribute_list {
            if !attribute.path.is_ident("name") {
                continue;
            }

            let parsed_attribute = match attribute.parse_meta() {
                Ok(meta) => meta,
                Err(err) => return err.to_compile_error(),
            };

            let name_value_attribute = match parsed_attribute {
                syn::Meta::NameValue(name_value_meta) => name_value_meta,
                _ => {
                    let token_span = attribute.span();
                    return quote_spanned! { token_span =>
                        compile_error!("Expected a value to be assigned to name");
                    };
                }
            };

            let name = match name_value_attribute.lit {
                syn::Lit::Str(name) => name.value(),
                _ => {
                    let name_token = name_value_attribute.span();
                    return quote_spanned! { name_token =>
                        compile_error!("name can only be assigned a string literal");
                    };
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

    element_creation
}
