mod utility;
mod execute;
mod query;
mod sql;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericParam, Ident, ItemStruct, Lifetime, LifetimeParam, LitStr, Type, parse_macro_input, parse_quote};

use crate::{ execute::Execute, query::Query};

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


#[proc_macro_attribute]
pub fn lazy_sql(_args: TokenStream, input: TokenStream) -> TokenStream {
    let mut item_struct = parse_macro_input!(input as ItemStruct);

    // 1. INJECT LIFETIME <'a>
    let lifetime_name = Lifetime::new("'a", Span::call_site());
    let lifetime_param = GenericParam::Lifetime(LifetimeParam::new(lifetime_name.clone()));
    item_struct.generics.params.insert(0, lifetime_param);

    // 2. INJECT __db FIELD
    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        let db_field: syn::Field = parse_quote! {
            __db: &#lifetime_name rsql::internal_sqlite::efficient::lazy_connection::LazyConnection
        };
        fields.named.insert(0, db_field);
    } else {
        return quote! { compile_error!("lazy_sql requires named fields"); }.into();
    }

    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();
    let struct_name = &item_struct.ident;

    let mut sql_assignments = Vec::new();
    let mut standard_assignments = Vec::new();
    let mut standard_params = Vec::new();
    let mut generated_methods = Vec::new();

    if let syn::Fields::Named(ref mut fields) = item_struct.fields {
        // Skip __db
        for field in fields.named.iter_mut().skip(1) {
            let ident = field.ident.as_ref().unwrap();
            
            // 3. DETECT `sql!("...")` IN THE TYPE POSITION
            let mut found_sql: Option<LitStr> = None;

            if let Type::Macro(type_macro) = &field.ty {
                // Check if the macro is `sql!`
                if type_macro.mac.path.is_ident("sql") {
                    // Parse the content of the macro as a string literal
                    match syn::parse2::<LitStr>(type_macro.mac.tokens.clone()) {
                        Ok(lit) => found_sql = Some(lit),
                        Err(_) => return quote! { compile_error!("sql!(...) must contain a string literal"); }.into(),
                    }
                }
            }

            if let Some(sql_lit) = found_sql {
                // --- IT IS A SQL FIELD ---

                // A. Change the type in the struct definition from `sql!(...)` to `LazyStmt`
                field.ty = parse_quote! { 
                    rsql::internal_sqlite::efficient::lazy_statement::LazyStmt 
                };

                // B. Generate the initializer for new()
                sql_assignments.push(quote! {
                    #ident: rsql::internal_sqlite::efficient::lazy_statement::LazyStmt {
                        sql_query: #sql_lit, // <--- We use the string we found
                        stmt: std::ptr::null_mut(),
                    }
                });

                // C. Generate the method
                generated_methods.push(quote! {
                    pub fn #ident(&mut self) -> Result<rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt, rsql::errors::connection::SqlitePrepareErrors> {
                        if self.#ident.stmt.is_null() {
                            unsafe { 
                                rsql::utility::utils::prepare_stmt(
                                    self.__db.db, 
                                    &mut self.#ident.stmt, 
                                    self.#ident.sql_query
                                )?; 
                            }
                        }
                        Ok(rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
                            stmt: self.#ident.stmt,
                            conn: self.__db.db,
                        })
                    }
                });

            } else {
                // --- IT IS A STANDARD FIELD ---
                let ty = &field.ty;
                standard_params.push(quote! { #ident: #ty });
                standard_assignments.push(quote! { #ident });
            }
        }
    }

    // 4. GENERATE OUTPUT
    let output = quote! {
        #item_struct

        impl #impl_generics #struct_name #ty_generics #where_clause {
            pub fn new(
                db: &#lifetime_name rsql::internal_sqlite::efficient::lazy_connection::LazyConnection, 
                #(#standard_params,)*
            ) -> Self {
                Self {
                    __db: db,
                    #(#standard_assignments,)* 
                    #(#sql_assignments,)*
                }
            }

            #(#generated_methods)*
        }
    };

    output.into()
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
            let #field_name = unsafe
            {
            <#field_type as rsql::traits::from_sql::FromSql>::from_sql(stmt, #index)
            };

        }
    });

    let field_names = fields.iter().map(|f| f.ident.as_ref().unwrap());
    let expanded = quote! {

        struct #mapper_struct_name;

        impl rsql::traits::row_mapper::RowMapper for #mapper_struct_name {
            type Output = #struct_name;

            unsafe fn map_row(&self, stmt: *mut libsqlite3_sys::sqlite3_stmt) -> Self::Output {
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
