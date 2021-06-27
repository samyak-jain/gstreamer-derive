use heck::SnakeCase;
use syn::Ident;

pub fn convert_ident_case(ident: &Ident) -> Ident {
    let ident_name = ident.to_string();
    let case_changed_name = ident_name.to_snake_case();

    Ident::new(&case_changed_name, ident.span())
}
