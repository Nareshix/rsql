use quote::{ToTokens, quote};
use syn::{
    Expr, ExprTuple, Path, Token,
    parse::{Parse, ParseStream},
};

use proc_macro2::TokenStream as TokenStream2;

//TODO only strictly allow Connection structs rather than a generic Path
pub struct Execute {
    pub conn: Path,
    pub sql_statement: Expr,
    pub sql_bindings: ExprTuple,
}

impl Parse for Execute {
    //TODO allow for optional tuples. shouldnt be hard
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let conn = input.parse()?;
        input.parse::<Token![,]>()?;

        let sql_statement = input.parse()?;
        input.parse::<Token![,]>()?;

        let sql_bindings = input.parse()?;

        // allow optilnal trailing commas
        if input.peek(Token![,]) {
            input.parse::<Token![,]>()?;
        }

        Ok(Execute {
            conn,
            sql_statement,
            sql_bindings,
        })
    }
}

impl ToTokens for Execute {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let conn = &self.conn;
        let sql_statement = &self.sql_statement;
        let sql_bindings = &self.sql_bindings;

        let generated_code = if sql_bindings.elems.is_empty() {
            quote! {
                // TODO SHOULD WE STEP?
                #conn.prepare(#sql_statement)?.step()
            }
        } else {
            let bindings = sql_bindings.elems.iter();

            // SQLite binding starts from 1-index
            let indices = 1..=bindings.len() as i32;

            quote! {
            // TODO should we use a block to keep `stmt` local
            let stmt = #conn.prepare(#sql_statement)?;
            #(stmt.bind_parameter(#indices, #bindings)?;)*
            stmt.step()

            }
        };

        tokens.extend(generated_code);
    }
}
