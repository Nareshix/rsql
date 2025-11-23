use proc_macro2::TokenStream as TokenStream2;
use quote::{quote, quote_spanned, ToTokens};
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
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
        let output_struct = &self.struct_def;
        let struct_name = &output_struct.ident;
        let (impl_generics, ty_generics, where_clause) = output_struct.generics.split_for_impl();

        // 1. Validate that we have named fields
        let fields = match &output_struct.fields {
            syn::Fields::Named(f) => &f.named,
            _ => {
                tokens.extend(quote_spanned! {
                    output_struct.span() => compile_error!("AutoStmt can only be used on structs with named fields.");
                });
                return;
            }
        };

        let mut db_field_ident = None;
        let mut new_assignments = Vec::new();
        let mut new_params = Vec::new();
        let mut generated_methods = Vec::new();

        // 2. Iterate fields once
        for field in fields {
            let ident = field.ident.as_ref().unwrap();
            let ty = &field.ty;

            // Check for #[sql("...")]
            let mut sql_lit: Option<LitStr> = None;
            for attr in &field.attrs {
                if attr.path().is_ident("sql") {
                    match attr.parse_args::<LitStr>() {
                        Ok(lit) => sql_lit = Some(lit),
                        Err(e) => {
                            tokens.extend(e.to_compile_error());
                            return;
                        }
                    }
                }
            }

            if let Some(sql) = sql_lit {
                // --- Case A: It is a SQL LazyStmt field ---
                
                // 1. Assignment in new()
                new_assignments.push(quote! {
                    #ident: LazyStmt {
                        sql_query: #sql,
                        stmt: std::ptr::null_mut(),
                    }
                });

                // 2. We defer method generation slightly until we confirm we found a DB field
                // We store a closure/data to generate it later, or we rely on db_field_ident being found.
                // For simplicity in this snippet, we assume db_field is available or we panic safely later.
                
            } else {
                // --- Case B: It is a normal field (likely the DB or config) ---
                
                // Simple heuristic: If it's named "db" OR we haven't found a DB field yet, assume this is it.
                // ideally, you should look for a #[db] attribute here for safety.
                if db_field_ident.is_none() {
                    db_field_ident = Some(ident.clone());
                }

                new_params.push(quote! { #ident: #ty });
                new_assignments.push(quote! { #ident });
            }
        }

        // 3. Validations
        let db_field_ident = match db_field_ident {
            Some(id) => id,
            None => {
                tokens.extend(quote_spanned! {
                    output_struct.span() => compile_error!("Could not determine which field is the database connection. Please ensure one field does not have #[sql].");
                });
                return;
            }
        };

        // 4. Generate Methods (Now that we have the db_ident)
        for field in fields {
            let ident = field.ident.as_ref().unwrap();
            
            // Re-check attributes (or you could have stored this in the loop above)
            let has_sql = field.attrs.iter().any(|a| a.path().is_ident("sql"));

            if has_sql {
                 generated_methods.push(quote! {
                    pub fn #ident(&mut self) -> Result<rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt, rsql::errors::connection::SqlitePrepareErrors> {
                        if self.#ident.stmt.is_null() {
                            unsafe { 
                                rsql::utility::utils::prepare_stmt(
                                    self.#db_field_ident.db, 
                                    &mut self.#ident.stmt, 
                                    self.#ident.sql_query
                                )?; 
                            }
                        }
                        Ok(rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
                            stmt: self.#ident.stmt,
                            conn: self.#db_field_ident.db,
                        })
                    }
                });
            }
        }

        // 5. Final Output
        // Note: We must strip the #[sql] attributes from the struct definition output
        // helper function to strip attributes
        let mut clean_struct = output_struct.clone();
        if let syn::Fields::Named(ref mut f) = clean_struct.fields {
             for field in &mut f.named {
                 field.attrs.retain(|a| !a.path().is_ident("sql"));
             }
        }

        tokens.extend(quote! {
            #clean_struct

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