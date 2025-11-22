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

        let mut new_params = Vec::new();
        let mut new_assignments = Vec::new();
        let mut generated_methods = Vec::new();

        let fields = match &mut output_struct.fields {
            syn::Fields::Named(f) => f,
            _ => {
                tokens.extend(quote_spanned! {
                    struct_name.span() => compile_error!("AutoStmt requires named fields");
                });
                return;
            }
        };

        for field in fields.named.iter_mut() {
            let ident = &field.ident;
            let ty = &field.ty; // This is likely 'LazyStmt'

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
                // 1. Constructor assignment
                new_assignments.push(quote! {
                    #ident: LazyStmt {
                        sql_query: #sql,
                        stmt: std::ptr::null_mut(),
                    }
                });

                // 2. Generated Method with your specific logic
                generated_methods.push(quote! {
                    pub fn #ident(&mut self) -> Result<&mut #ty, rsql::errors::connection::SqlitePrepareErrors> {
                        if self.#ident.stmt.is_null() {
                            unsafe { 
                                prepare_stmt(
                                    self.db.db, 
                                    &mut self.#ident.stmt, 
                                    self.#ident.sql_query
                                )?; 
                            }
                        }
                        Ok(&mut self.#ident)
                    }
                });
            } else {
                // Standard fields
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