use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput};

use crate::attributes::{link::generate_links, name::create_elements};

mod attributes;
mod utils;

#[proc_macro_derive(GStreamer, attributes(name, link_many))]
pub fn gstreamer_maker(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed_input = parse_macro_input!(input as DeriveInput);
    let expanded = gstreamer_implementation(parsed_input);

    expanded.into()
}

fn gstreamer_implementation(input: DeriveInput) -> TokenStream {
    let name = &input.ident;

    let generated_name = format_ident!("__GStreamer{}", name);

    let created_elements = match input.data {
        syn::Data::Enum(ref enum_value) => create_elements(enum_value),
        _ => {
            let data_span = input.span();
            quote_spanned! { data_span =>
                compile_error!("This macro only works with enums");
            }
        }
    };

    let element_variables = match input.data {
        syn::Data::Enum(ref enum_value) => enum_value
            .variants
            .iter()
            .map(|variant| utils::convert_ident_case(&variant.ident))
            .collect::<Vec<Ident>>(),
        _ => {
            let data_span = input.span();
            return quote_spanned! { data_span =>
                compile_error!("This macro only works with enums");
            };
        }
    };

    let generated_links = generate_links(&input.attrs, &element_variables);

    quote! {
        struct #generated_name {
            pipeline: gstreamer::Pipeline,
            #(#element_variables: gstreamer::Element),*
        }

        impl #name {
            fn build() -> #generated_name {
                use gstreamer::prelude::*;

                #created_elements

                let pipeline = gstreamer::Pipeline::new(Some(stringify!(#name)));
                pipeline.add_many(&[
                    #(&#element_variables),*
                ]);

                #generated_links

                #generated_name {
                    pipeline,
                    #(#element_variables),*
                }
            }
        }

        impl #generated_name {
            fn start(&self) {
                use gstreamer::prelude::ElementExt;

                self.pipeline.set_state(gstreamer::State::Playing)
                    .expect("Unable to set the pipeline to the `Playing` state");
            }

            fn stop(&self) {
                use gstreamer::prelude::ElementExt;

                self.pipeline.set_state(gstreamer::State::Null)
                    .expect("Unable to set the pipeline to the `Null` state");
            }
        }

        impl Drop for #generated_name {
            fn drop(&mut self) {
                self.stop();
            }
        }
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
