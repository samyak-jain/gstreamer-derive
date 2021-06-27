use proc_macro2::Ident;
use quote::{quote, quote_spanned};
use syn::{spanned::Spanned, Attribute, NestedMeta};

fn get_ident<'a>(token: &'a NestedMeta, ident_list: &[Ident]) -> Option<&'a Ident> {
    if let NestedMeta::Meta(meta) = token {
        let path = meta.path();
        let ident = path.get_ident();
        ident.and_then(|ident| {
            if !ident_list.contains(&ident) {
                return None;
            }
            Some(ident)
        })
    } else {
        None
    }
}

pub fn generate_links(
    attribute_list: &[Attribute],
    ident_list: &[Ident],
) -> proc_macro2::TokenStream {
    let mut link_creation = quote!();

    for attribute in attribute_list {
        if !attribute.path.is_ident("link") {
            continue;
        }

        let parsed_attribute = match attribute.parse_meta() {
            Ok(meta) => meta,
            Err(err) => return err.to_compile_error(),
        };

        let list_attribute = match parsed_attribute {
            syn::Meta::List(list_meta) => list_meta,
            _ => {
                let token_span = attribute.span();
                return quote_spanned! { token_span =>
                    compile_error!("Expected a value to be assigned to name");
                };
            }
        };

        if list_attribute.nested.len() < 2 {
            let attribute_span = list_attribute.nested.span();
            return quote_spanned! { attribute_span =>
                    compile_error!("You need to specify atleast 2 elements when you are linking");
            };
        }

        let mut nested_list_attribute_iterator = list_attribute.nested.iter();
        let mut previous_list_element = match nested_list_attribute_iterator.next() {
            Some(element) => element,
            None => unreachable!(),
        };

        for current_list_element in nested_list_attribute_iterator {
            let prev_literal = get_ident(previous_list_element, ident_list);
            let current_literal = get_ident(current_list_element, ident_list);

            previous_list_element = current_list_element;

            match (prev_literal, current_literal) {
                (Some(left_link), Some(right_link)) => {
                    link_creation.extend(quote! {
                        #left_link.link(&#right_link).unwrap();
                    });
                }
                _ => continue,
            };
        }
    }

    link_creation
}
