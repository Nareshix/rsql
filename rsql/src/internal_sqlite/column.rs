// use libsqlite3_sys::sqlite3_step;

// use crate::internal_sqlite::statement::Statement;

// struct Row<'a> {
//     stmt: Statement<'a>
// }

// impl Iterator for Row<'_> {

//     fn next(&mut self) -> Option<Self::Item> {
//         unsafe {sqlite3_step(self.stmt.stmt)}
//     }
// }
// // pub fn query(&self) {
//     //     // let code = self.step();

//     //     // loop through the rows
//     //     while self.step() == SQLITE_ROW {
//     //         let no_of_columns = unsafe { sqlite3_column_count(self.stmt) };

//     //         // loop through each columns
//     //         for col_index in 0..no_of_columns - 1 {
//     //             let sql_col_type_code = unsafe { sqlite3_column_type(self.stmt, col_index) };
            
//     //             match sql_col_type_code {
//     //                 //TODO prins out for each case
//     //             }
//     //             //TODO unwrap
//     //             // let sql_col_type = sqlite_to_rust_type_mapping(sql_col_type_code).unwrap();

//     //         }
//     //     }
//     // }
