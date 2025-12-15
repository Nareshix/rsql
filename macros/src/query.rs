use quote::{ToTokens, quote};
use syn::{
    Expr, ExprTuple, Path, Token,
    parse::{Parse, ParseStream},
};

use proc_macro2::TokenStream as TokenStream2;

//TODO only strictly allow Connection structs rather than a generic Path
pub struct Query {
    pub conn: Path,
    pub sql_mapping: Path,
    pub sql_statement: Expr,
    pub sql_bindings: ExprTuple,
}

impl Parse for Query {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let conn = input.parse()?;
        input.parse::<Token![,]>()?;

        let sql_mapping = input.parse()?;
        input.parse::<Token![,]>()?;

        let sql_statement = input.parse()?;
        input.parse::<Token![,]>()?;

        let sql_bindings = input.parse()?;

        // allow optilnal trailing commas
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(Query {
            conn,
            sql_mapping,
            sql_statement,
            sql_bindings,
        })
    }
}

impl ToTokens for Query {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let conn = &self.conn;
        let sql_mapping = &self.sql_mapping;
        let sql_statement = &self.sql_statement;
        let sql_bindings = &self.sql_bindings;

        let generated_code = if sql_bindings.elems.is_empty() {
            quote! {
                {
                    let stmt = #conn.prepare(#sql_statement)?;
                    stmt.query(#sql_mapping).collect::<Vec<_>>()
                }
            }
        } else {
            let bindings = sql_bindings.elems.iter();

            // SQLite binding starts from 1-index
            let indices = 1..=bindings.len() as i32;

            quote! {
                {
                    let stmt = #conn.prepare(#sql_statement)?;
                    #(stmt.bind_parameter(#indices, #bindings)?;)*
                    stmt.query(#sql_mapping).collect::<Vec<_>>()
                }
            }
        };

        tokens.extend(generated_code);
    }
}
