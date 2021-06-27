use proc_macro2::{Ident, TokenStream};
use quote::{format_ident, quote, quote_spanned};
use syn::{parse_macro_input, spanned::Spanned, DeriveInput, Lit};

use crate::attributes::name::create_elements;

mod attributes;

#[proc_macro_derive(GStreamer, attributes(name, link_many))]
pub fn gstreamer_maker(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parsed_input = parse_macro_input!(input as DeriveInput);
    let expanded = gstreamer_implementation(parsed_input);

    expanded.into()
}

fn gstreamer_implementation(input: DeriveInput) -> TokenStream {
    let name = input.ident;
    let global_attrs = input.attrs;
    let mut links: Vec<Vec<Ident>> = Vec::new();

    for global_attribute in global_attrs {
        if let syn::Meta::List(parsed_global_attrs) = global_attribute.parse_meta().unwrap() {
            let path = parsed_global_attrs.path;
            if path.is_ident("link") {
                links.push(
                    parsed_global_attrs
                        .nested
                        .into_iter()
                        .filter_map(|nested_path| {
                            if let syn::NestedMeta::Meta(meta) = nested_path {
                                return meta.path().get_ident().map(|ident| ident.to_owned());
                            }

                            None
                        })
                        .collect(),
                );
            }
        }
    }

    let mut generated_links: Vec<proc_macro2::TokenStream> = Vec::new();
    for link in links {
        let mut token_vec = Vec::with_capacity(link.len());
        let mut link_iterator = link.iter();
        let mut prev_link = match link_iterator.next() {
            Some(plink) => plink,
            None => continue,
        };

        while let Some(next_link) = link_iterator.next() {
            token_vec.push(quote! {
                #prev_link.link(&#next_link).unwrap();
            });
            prev_link = next_link;
        }

        generated_links.extend(token_vec);
    }

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
            .map(|variant| &variant.ident)
            .collect::<Vec<&Ident>>(),
        _ => {
            let data_span = input.span();
            return quote_spanned! { data_span =>
                compile_error!("This macro only works with enums");
            };
        }
    };

    quote! {
        use gstreamer::Element;

        struct #generated_name {
            pipeline: gstreamer::Pipeline,
            #(pub #element_variables: Element),*
        }

        impl #name {
            fn build() -> #generated_name {
                #created_elements

                let pipeline = gstreamer::Pipeline::new(Some(stringify!(#name)));
                pipeline.add_many(&[
                    #(&#element_variables),*
                ]);

                #(#generated_links)*

                #generated_name {
                    pipeline,
                    #(#element_variables),*
                }
            }
        }

        impl #generated_name {
            fn start(&self) {
                self.pipeline.set_state(gstreamer::State::Playing).expect("Unable to set the pipeline to the `Playing` state");
            }

            fn stop(&self) {
                self.pipeline.set_state(gstreamer::State::Null).expect("Unable to set the pipeline to the `Null` state");
            }
        }

        impl Drop for #generated_name {
            fn drop(&self) {
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
