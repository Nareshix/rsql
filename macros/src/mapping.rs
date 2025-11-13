// use quote::{ToTokens, quote};
// use syn::{
//     Expr, ExprTuple, Path, Token,
//     parse::{Parse, ParseStream},
// };

// use proc_macro2::TokenStream as TokenStream2;

// struct Mapping;


// // impl ToTokens for Mapping {
// //     fn to_tokens(&self, tokens: &mut TokenStream2) {
// //         let conn = &self.conn;
// //         let sql_statement = &self.sql_statement;
// //         let sql_bindings = &self.sql_bindings;

// //         let generated_code = if sql_bindings.elems.is_empty() {
// //             quote! {
// //                 // TODO SHOULD WE STEP?
// //                 #conn.prepare(#sql_statement)?.step()
// //             }
// //         } else {
// //             let bindings = sql_bindings.elems.iter();

// //             // SQLite binding starts from 1-index
// //             let indices = 1..=bindings.len() as i32;

// //             quote! {
// //             // TODO should we use a block to keep `stmt` local
// //             let stmt = #conn.prepare(#sql_statement)?;
// //             #(stmt.bind_parameter(#indices, #bindings)?;)*
// //             stmt.step()

// //             }
// //         };

// //         tokens.extend(generated_code);
// //     }
// // }

