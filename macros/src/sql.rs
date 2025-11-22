use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    ItemStruct, LitStr,
};

pub struct AutoStmt {
    pub struct_def: ItemStruct,
}

impl Parse for AutoStmt {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(AutoStmt {
            struct_def: input.parse()?,
        })
    }
}
impl ToTokens for AutoStmt {
    fn to_tokens(&self, tokens: &mut TokenStream2) {
        let mut output_struct = self.struct_def.clone();
        let struct_name = &output_struct.ident;
        let (impl_generics, ty_generics, where_clause) = output_struct.generics.split_for_impl();

        let db_field_ident = match &self.struct_def.fields {
            syn::Fields::Named(fields) => fields.named.iter().find_map(|f| {
                let has_sql = f.attrs.iter().any(|attr| attr.path().is_ident("sql"));
                if !has_sql {
                    f.ident.clone()
                } else {
                    None
                }
            }).expect("Struct must have a field for the database connection (a field without #[sql])"),
            _ => panic!("AutoStmt requires named fields"),
        };

        let mut new_params = Vec::new();
        let mut new_assignments = Vec::new();
        let mut generated_methods = Vec::new();

        let fields = match &mut output_struct.fields {
            syn::Fields::Named(f) => f,
            _ => {
                // This error is theoretically unreachable due to the check above, but safe to keep
                tokens.extend(quote_spanned! {
                    struct_name.span() => compile_error!("AutoStmt requires named fields");
                });
                return;
            }
        };

        for field in fields.named.iter_mut() {
            let ident = &field.ident;
            let ty = &field.ty;

            let mut sql_lit: Option<LitStr> = None;
            let mut parse_error = None;

            // Extract 'sql' attribute
            field.attrs.retain(|attr| {
                if attr.path().is_ident("sql") {
                    match attr.parse_args::<LitStr>() {
                        Ok(lit) => sql_lit = Some(lit),
                        Err(e) => parse_error = Some(e),
                    }
                    false
                } else {
                    true
                }
            });

            if let Some(err) = parse_error {
                tokens.extend(err.to_compile_error());
                return;
            }

            if let Some(sql) = sql_lit {
                // 1. Constructor assignment for LazyStmt
                new_assignments.push(quote! {
                    #ident: LazyStmt {
                        sql_query: #sql,
                        stmt: std::ptr::null_mut(),
                    }
                });

                // 2. Generated Method
                generated_methods.push(quote! {
                    pub fn #ident(&mut self) -> Result<rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt, rsql::errors::connection::SqlitePrepareErrors> {
                        
                        if self.#ident.stmt.is_null() {
                            unsafe { 
                                // --- FIX START: 2. Use #db_field_ident instead of "db" ---
                                rsql::utility::utils::prepare_stmt(
                                    self.#db_field_ident.db, // Was: self.db.db
                                    &mut self.#ident.stmt, 
                                    self.#ident.sql_query
                                )?; 
                            }
                        }

                        Ok(rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
                            stmt: self.#ident.stmt,
                            conn: self.#db_field_ident.db, // Was: self.db.db
                        })
                        // --- FIX END ---
                    }
                });
            } else {
                // Standard fields (The DB Connection)
                new_params.push(quote! { #ident: #ty });
                new_assignments.push(quote! { #ident });
            }
        }

        tokens.extend(quote! {
            #output_struct

            impl #impl_generics #struct_name #ty_generics #where_clause {
                pub fn new( #(#new_params),* ) -> Self {
                    Self {
                        #(#new_assignments),*
                    }
                }

                #(#generated_methods)*
            }
        });
    }
}