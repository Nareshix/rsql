mod execute;
mod query;
mod compile_time_check;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Fields, GenericParam, Ident, ItemStruct, Lifetime, LifetimeParam, LitStr, Type, parse_macro_input, parse_quote, spanned::Spanned};

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

    match expand(&mut item_struct) {
        Ok(output) => output.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn expand(item_struct: &mut ItemStruct) -> syn::Result<proc_macro2::TokenStream> {
    let struct_name = &item_struct.ident;

    // 1. Validate it has named fields
    let fields = match &mut item_struct.fields {
        syn::Fields::Named(named) => named,
        _ => return Err(syn::Error::new(
            item_struct.span(),
            "lazy_sql requires a struct with named fields",
        )),
    };

    let mut sql_assignments = Vec::new();
    let mut standard_assignments = Vec::new();
    let mut standard_params = Vec::new();
    let mut generated_methods = Vec::new();

    // 2. Iterate over existing fields (Before injecting __db)
    for field in fields.named.iter_mut() {
        let ident = field.ident.as_ref().unwrap();

        // Check if type is sql!("...")
        if let Some(sql_lit) = parse_sql_macro_type(&field.ty) {
            let sql_query = sql_lit.value();
            let doc_comment = format!(" **SQL**\n```sql\n{}", sql_query);
            
            // --- IS SQL FIELD ---
            
            // A. Replace type with LazyStmt
            field.ty = parse_quote!(rsql::internal_sqlite::efficient::lazy_statement::LazyStmt);

            // B. Initializer
            sql_assignments.push(quote! {
                #ident: rsql::internal_sqlite::efficient::lazy_statement::LazyStmt {
                    sql_query: #sql_lit,
                    stmt: std::ptr::null_mut(),
                }
            });

            // C. Method Generation
            generated_methods.push(quote! {
                #[doc = #doc_comment]
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
            // --- IS STANDARD FIELD ---
            let ty = &field.ty;
            standard_params.push(quote! { #ident: #ty });
            standard_assignments.push(quote! { #ident });
        }
    }

    // 3. INJECT LIFETIME <'a>
    // parse_quote! makes this much more readable than manual construction
    let lifetime_def: LifetimeParam = parse_quote!('a);
    item_struct.generics.params.insert(0, GenericParam::Lifetime(lifetime_def));

    // 4. INJECT __db FIELD
    // We do this LAST so we didn't have to skip(1) in the loop above
    fields.named.insert(0, parse_quote! {
        __db: &'a rsql::internal_sqlite::efficient::lazy_connection::LazyConnection
    });

    let (impl_generics, ty_generics, where_clause) = item_struct.generics.split_for_impl();

    // 5. Generate the final code
    Ok(quote! {
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
    })
}

/// Helper to extract the string literal from `sql!("SELECT...")`
fn parse_sql_macro_type(ty: &Type) -> Option<LitStr> {
    if let Type::Macro(type_macro) = ty {
        if type_macro.mac.path.is_ident("sql") {
            // syn::parse2 is powerful: it takes a TokenStream and parses it into a Node
            return syn::parse2::<LitStr>(type_macro.mac.tokens.clone()).ok();
        }
    }
    None
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
