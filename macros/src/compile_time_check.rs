use quote::{ToTokens, quote};
use rsql_core::internal_sqlite::connection::Connection;
use syn::{
    LitStr, parse::{Parse, ParseStream},
};

use proc_macro2::TokenStream as TokenStream2;

pub struct Check {
    pub sql: LitStr,
}

impl Parse for Check {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let sql = input.parse()?;

        Ok(Check { sql })
    }
}
impl ToTokens for Check {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let sql_literal = &self.sql;
        let sql_string = sql_literal.value(); // Get the actual string value

        // 1. Connect to the database AT COMPILE TIME.
        //    Note: The database file ("hi.db") must exist where you run `cargo build`.
        let conn = match Connection::open("hi.db") {
            Ok(c) => c,
            Err(e) => {
                // If we can't even open the DB, create a compile error.
                let err = syn::Error::new(
                    sql_literal.span(),
                    format!("Failed to open database at compile time: {}", e),
                )
                .to_compile_error();
                tokens.extend(err);
                return;
            }
        };

        // 2. Try to prepare the SQL statement.
        match conn.prepare(&sql_string) {
            // 3. If it succeeds, the SQL is valid!
            //    Expand the macro to the original string literal.
            Ok(_) => {
                tokens.extend(quote! { #sql_literal });
            }
            // 4. If it fails, the SQL is invalid.
            //    Generate a specific compile error.
            Err(e) => {
                let error_message = format!("Invalid SQL: {}", e);
                let err = syn::Error::new(sql_literal.span(), error_message).to_compile_error();
                tokens.extend(err);
            }
        }
    }
}