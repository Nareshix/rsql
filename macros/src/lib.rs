mod execute;
mod mapping;
mod query;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, Ident, parse_macro_input};

use crate::{execute::Execute, query::Query};

#[proc_macro]
pub fn execute(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as Execute);
    quote! { #parsed_input }.into()
}

#[proc_macro]
pub fn query(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as Query);
    quote! { #parsed_input }.into()
}


#[proc_macro_derive(SqlMapping)]
pub fn my_macro(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let struct_name = &input.ident;

    let name_as_string = struct_name.to_string();
    let new_name_string = format!("{}Mapper", name_as_string);
    let mapper_struct_name = Ident::new(&new_name_string, struct_name.span());

    // TODO error handling
    let fields = match &input.data {
        Data::Struct(s) => match &s.fields {
            Fields::Named(fields_named) => &fields_named.named,
            _ => panic!("This macro only works on structs with named fields"),
        },
        _ => panic!("This macro only works on structs"),
    };

    let field_bindings = fields.iter().enumerate().map(|(i, f)| {
        let field_name = f.ident.as_ref().unwrap();
        let field_type = &f.ty;
        let index = i as i32;

        quote! {
            let #field_name = unsafe { #field_type::from_col(stmt, #index) };
        }
    });

    let field_names = fields.iter().map(|f| f.ident.as_ref().unwrap());
    let expanded = quote! {
        use libsqlite3_sys::sqlite3_stmt;
        use rsql::traits::row_mapper::RowMapper;
        use rsql::traits::from_sql::FromSql;


        struct #mapper_struct_name;

        impl RowMapper for #mapper_struct_name {
            type Output = #struct_name;

            unsafe fn map_row(&self, stmt: *mut sqlite3_stmt) -> Self::Output {
                #(#field_bindings)*

                Self::Output {
                    #(#field_names),*
                }
            }
        }

        #[allow(non_upper_case_globals)]
        const #struct_name: #mapper_struct_name = #mapper_struct_name;
    };

    TokenStream::from(expanded)
}
