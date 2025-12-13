mod execute;
mod query;
mod utils;

use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use rsql_core::utility::utils::{get_db_schema, validate_sql_syntax_with_sqlite};
use syn::{
    Data, DeriveInput, Fields, GenericParam, Ident, ItemStruct, LifetimeParam, LitStr, Type,
    parse_macro_input, parse_quote, spanned::Spanned,
};
use type_inference::{
    binding_patterns::get_type_of_binding_parameters, expr::BaseType,
    pg_type_cast_to_sqlite::pg_cast_syntax_to_sqlite, select_patterns::get_types_from_select,
    table::create_tables,
};

use crate::{execute::Execute, query::Query, utils::format_sql};

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
pub fn lazy_sql(args: TokenStream, input: TokenStream) -> TokenStream {
    let path_lit = match syn::parse::<syn::LitStr>(args) {
        Ok(lit) => lit,
        Err(_) => {
            let err = syn::Error::new(
                proc_macro2::Span::call_site(),
                "lazy_sql requires a path argument, e.g., #[lazy_sql(\"db.sqlite\")]",
            );

            let err_tokens = err.to_compile_error();

            let input_tokens = proc_macro2::TokenStream::from(input);

            return quote! {
                #err_tokens
                #input_tokens
            }
            .into();
        }
    };

    let mut item_struct = parse_macro_input!(input as ItemStruct);

    match expand(&mut item_struct, &path_lit) {
        Ok(output) => output.into(),
        Err(err) => {
            let err_tokens = err.to_compile_error();
            err_tokens.into()
        }
    }
}

fn expand(
    item_struct: &mut ItemStruct,
    db_path_lit: &syn::LitStr,
) -> syn::Result<proc_macro2::TokenStream> {
    let db_path = db_path_lit.value();

    let schemas = get_db_schema(&db_path).map_err(|err| {
        syn::Error::new(
            db_path_lit.span(),
            format!("Failed to load DB schema: {}", err),
        )
    })?;

    let mut all_tables = HashMap::new();

    for schema in schemas {
        create_tables(&schema, &mut all_tables);
    }

    let struct_name = &item_struct.ident;

    let fields = match &mut item_struct.fields {
        syn::Fields::Named(named) => named,
        _ => {
            return Err(syn::Error::new(
                item_struct.span(),
                "lazy_sql requires a struct with named fields",
            ));
        }
    };

    let mut sql_assignments = Vec::new();
    let mut standard_assignments = Vec::new();
    let mut standard_params = Vec::new();
    let mut generated_methods = Vec::new();
    let mut generated_structs = Vec::new();
    let mut re_exports = Vec::new();

    for field in fields.named.iter_mut() {
        let ident = field.ident.as_ref().unwrap();

        // Check if type is sql!("...")
        if let Some(sql_lit) = parse_sql_macro_type(&field.ty)? {
            let sql_query = pg_cast_syntax_to_sqlite(&sql_lit.value());

            if let Err(err_msg) = validate_sql_syntax_with_sqlite(&db_path, &sql_query) {
                return Err(syn::Error::new(sql_lit.span(), err_msg.to_string()));
            }

            let select_types = match get_types_from_select(&sql_query, &all_tables) {
                Ok(types) => types,
                Err(err_msg) => {
                    return Err(syn::Error::new(
                        sql_lit.span(),
                        format!("Return Type Error: {}", err_msg),
                    ));
                }
            };

            let binding_types = match get_type_of_binding_parameters(&sql_query, &all_tables) {
                Ok(types) => types,
                Err(err) => {
                    let lines: Vec<&str> = sql_query.lines().collect();
                    let line_idx = err.start.line.saturating_sub(1) as usize;
                    let start_col = err.start.column.saturating_sub(1) as usize;
                    let end_col = err.end.column.saturating_sub(1) as usize;
                    let mut msg = format!("Parameter Binding Error: {}", err.message);
                    if let Some(raw_line) = lines.get(line_idx) {
                        let indent_len_bytes = raw_line
                            .char_indices()
                            .take_while(|(_, c)| c.is_whitespace())
                            .last()
                            .map(|(i, c)| i + c.len_utf8())
                            .unwrap_or(0);
                        let start_byte_idx = raw_line
                            .chars()
                            .take(start_col)
                            .map(|c| c.len_utf8())
                            .sum::<usize>();
                        let end_byte_idx = raw_line
                            .chars()
                            .take(end_col)
                            .map(|c| c.len_utf8())
                            .sum::<usize>();
                        let safe_indent = if indent_len_bytes <= start_byte_idx {
                            indent_len_bytes
                        } else {
                            0
                        };
                        let trimmed_line = &raw_line[safe_indent..];
                        let err_start_in_trimmed = start_byte_idx - safe_indent;
                        let err_len = end_byte_idx - start_byte_idx;
                        let padding: String = trimmed_line[..err_start_in_trimmed]
                            .chars()
                            .map(|c| if c == '\t' { '\t' } else { ' ' })
                            .collect();
                        let arrows = "^".repeat(err_len.max(1));
                        msg = format!("{}\n\n{}\n{}{}", msg, trimmed_line, padding, arrows);
                    }
                    return Err(syn::Error::new(sql_lit.span(), msg));
                }
            };

            let formated_sql_query = format_sql(&sql_query);
            let doc_comment = format!(" **SQL**\n```sql\n{}", formated_sql_query);

            field.ty = parse_quote!(rsql::internal_sqlite::efficient::lazy_statement::LazyStmt);

            sql_assignments.push(quote! {
                #ident: rsql::internal_sqlite::efficient::lazy_statement::LazyStmt {
                    sql_query: #sql_lit,
                    stmt: std::ptr::null_mut(),
                }
            });

            if select_types.is_empty() && binding_types.is_empty() {
                generated_methods.push(quote! {
                    #[doc = #doc_comment]
                    pub fn #ident(&mut self) -> Result<(), rsql::errors::SqlWriteError> {
                        if self.#ident.stmt.is_null() {
                            unsafe {
                                rsql::utility::utils::prepare_stmt(
                                    self.__db.db,
                                    &mut self.#ident.stmt,
                                    self.#ident.sql_query
                                )?;
                            }
                        }
                        let mut preparred_statement = rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
                            stmt: self.#ident.stmt,
                            conn: self.__db.db,
                        };
                        preparred_statement.step()?;
                        Ok(())
                    }
                });
            } else if select_types.is_empty() && !binding_types.is_empty() {
                let mut method_args = Vec::new();
                let mut bind_calls = Vec::new();

                for (i, bind_type) in binding_types.iter().enumerate() {
                    let arg_name = quote::format_ident!("arg_{}", i);
                    let bind_index = (i + 1) as i32;

                    let rust_base_type = match bind_type.base_type {
                        BaseType::Integer => quote! { i64 },
                        BaseType::Real => quote! { f64 },
                        BaseType::Bool => quote! { bool },
                        BaseType::Text => quote! { &str },
                        _ => quote! { Vec<u8> },
                    };

                    let final_type = if bind_type.nullable {
                        quote! { Option<#rust_base_type> }
                    } else {
                        quote! { #rust_base_type }
                    };

                    method_args.push(quote! { #arg_name: #final_type });

                    bind_calls.push(quote! {
                        preparred_statement.bind_parameter(#bind_index, #arg_name)?;
                    });
                }

                generated_methods.push(quote! {
                    #[doc = #doc_comment]
                    pub fn #ident(&mut self, #(#method_args),*) -> Result<(), rsql::errors::SqlWriteBindingError> {
                        if self.#ident.stmt.is_null() {
                            unsafe {
                                rsql::utility::utils::prepare_stmt(
                                    self.__db.db,
                                    &mut self.#ident.stmt,
                                    self.#ident.sql_query
                                )?;
                            }
                        }

                        let mut preparred_statement = rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
                            stmt: self.#ident.stmt,
                            conn: self.__db.db,
                        };

                        #(#bind_calls)*

                        preparred_statement.step()?;

                        Ok(())
                    }
                });
            } else if !select_types.is_empty() && binding_types.is_empty() {
                let method_name = ident.to_string();
                let pascal_name: String = method_name
                    .split('_')
                    .map(|s| {
                        let mut c = s.chars();
                        match c.next() {
                            None => String::new(),
                            Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
                        }
                    })
                    .collect();

                let struct_name = quote::format_ident!("{}", pascal_name);
                let mapper_name = quote::format_ident!("{}_", pascal_name);

                re_exports.push(struct_name.clone());
                re_exports.push(mapper_name.clone());

                let mut struct_fields = Vec::new();
                let mut mapper_bindings = Vec::new();
                let mut field_names = Vec::new();

                for (i, col) in select_types.iter().enumerate() {
                    let name = quote::format_ident!("{}", col.name);

                    let base_ty = match col.data_type.base_type {
                        BaseType::Integer => quote! { i64 },
                        BaseType::Real => quote! { f64 },
                        BaseType::Text => quote! { String },
                        BaseType::Bool => quote! { bool },
                        _ => quote! { Vec<u8> },
                    };

                    let final_ty = if col.data_type.nullable {
                        quote! { Option<#base_ty> }
                    } else {
                        quote! { #base_ty }
                    };

                    struct_fields.push(quote! { pub #name: #final_ty });

                    let index = i as i32;
                    mapper_bindings.push(quote! {
                        let #name = unsafe {
                            <#final_ty as rsql::traits::from_sql::FromSql>::from_sql(stmt, #index)
                        };
                    });

                    field_names.push(name);
                }

                generated_structs.push(quote! {
                    #[derive(Debug)]
                    pub struct #struct_name {
                        #(#struct_fields),*
                    }
                });

                generated_structs.push(quote! {
                    #[derive(Debug)]
                    pub struct #mapper_name;

                    impl rsql::traits::row_mapper::RowMapper for #mapper_name {
                        type Output = #struct_name;

                        unsafe fn map_row(&self, stmt: *mut libsqlite3_sys::sqlite3_stmt) -> Self::Output {
                            #(#mapper_bindings)*
                            Self::Output { #(#field_names),* }
                        }
                    }
                });

                generated_methods.push(quote! {
                    #[doc = #doc_comment]
                    pub fn #ident(&mut self) -> Result<rsql::internal_sqlite::efficient::rows_dao::Rows<#mapper_name>, rsql::errors::SqlReadError> {
                        if self.#ident.stmt.is_null() {
                            unsafe {
                                rsql::utility::utils::prepare_stmt(
                                    self.__db.db,
                                    &mut self.#ident.stmt,
                                    self.#ident.sql_query
                                )?;
                            }
                        }

                        let preparred_statement = rsql::internal_sqlite::efficient::preparred_statement::PreparredStmt {
                            stmt: self.#ident.stmt,
                            conn: self.__db.db,
                        };

                        Ok(preparred_statement.query(#mapper_name))
                    }
                });
            } else {
                //TODO
            }
        } else {
            let ty = &field.ty;
            standard_params.push(quote! { #ident: #ty });
            standard_assignments.push(quote! { #ident });
        }
    }

    let lifetime_def: LifetimeParam = parse_quote!('a);
    item_struct
        .generics
        .params
        .insert(0, GenericParam::Lifetime(lifetime_def));

    fields.named.insert(
        0,
        parse_quote! {
            __db: &'a rsql::internal_sqlite::efficient::lazy_connection::LazyConnection
        },
    );

    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();

    let mod_name = quote::format_ident!(
        "__lazy_sql_inner_{}",
        struct_name.to_string().to_lowercase()
    );

    item_struct.vis = parse_quote!(pub);

    Ok(quote! {
        #[doc(hidden)]
        mod #mod_name {
            use super::*;
            #(#generated_structs)*
            #item_struct

            impl #impl_generics #struct_name #ty_generics #where_clause {
                pub fn new(
                    db: &'a rsql::internal_sqlite::efficient::lazy_connection::LazyConnection,
                    #(#standard_params),*
                ) -> Self {
                    Self {
                        __db: db,
                        #(#standard_assignments,)*
                        #(#sql_assignments,)*
                    }
                }

                #(#generated_methods)*
            }
        }

        pub use #mod_name::#struct_name;
    })
}

fn parse_sql_macro_type(ty: &Type) -> syn::Result<Option<LitStr>> {
    if let Type::Macro(type_macro) = ty
        && type_macro.mac.path.is_ident("sql")
    {
        let lit = syn::parse2(type_macro.mac.tokens.clone()).map_err(|_| {
            syn::Error::new(
                type_macro.mac.tokens.span(),
                "sql!(...) must contain a string",
            )
        })?;

        return Ok(Some(lit));
    }

    Ok(None)
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
        // 1. Make the Mapper struct public
        #[derive(Clone, Copy, Debug)]
        pub struct #mapper_struct_name;

        impl rsql::traits::row_mapper::RowMapper for #mapper_struct_name {
            type Output = #struct_name;

            unsafe fn map_row(&self, stmt: *mut libsqlite3_sys::sqlite3_stmt) -> Self::Output {
                #(#field_bindings)*

                Self::Output {
                    #(#field_names),*
                }
            }
        }

        // 2. Make the const instance public
        #[allow(non_upper_case_globals)]
        pub const #struct_name: #mapper_struct_name = #mapper_struct_name;
    };

    TokenStream::from(expanded)
}
