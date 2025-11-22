// use super::connection::Connection;
// use std::fs;
// use std::path::Path;

// #[test]
// fn test_open_memory_db() {
//     let result = Connection::open_memory();
//     assert!(result.is_ok());
// }

// #[test]
// fn test_open_file_db() {
//     // long random name to avoid potential clash
//     let db_path = "test_12321o3ij1k2hj321hfdsffsgdsfgdsgffdsgsdgfflhfl1hr3l23.db";
    
//     Connection::open(db_path).expect("failed to opoen db");

//     assert!(Path::new(db_path).exists());
//     let _ = fs::remove_file(db_path);
// }



// #[test]
// fn test_prepare_statement() {
//     let conn = Connection::open_memory().expect("Failed to open Connection");
//     let sql = "CREATE TABLE test (id INTEGER PRIMARY KEY);";
    
//     let prepare_result = conn.prepare(sql);
//     assert!(prepare_result.is_ok());
// }



// #[test]
// fn test_prepare_statements_with_question_mark_parameter() {
//     // eh, for now just use the file db eventually use memory db
//     let conn = Connection::open("asd.db").expect("Failed to open Connection");
//     let stmt = conn
//         .prepare("SELECT * FROM test WHERE name = ?");

//     assert!(stmt.is_ok())
// }


// #[test]
// fn test_prepare_statements_with_multiple_question_mark_parameter() {
//     // eh, for now just use the file db eventually use memory db
//     let conn = Connection::open("asd.db").expect("Failed to open Connection");
//     let stmt = conn
//         .prepare("SELECT * FROM test WHERE name = ? AND id=?");

//     assert!(stmt.is_ok())
// }


// #[test]
// fn test_prepare_statements_with_multiple_dollar_number_parameter() {
//     let conn = Connection::open("asd.db").expect("Failed to open Connection");
//     let stmt = conn
//         .prepare("SELECT * FROM test WHERE name = $1 AND id= $2");

//     assert!(stmt.is_ok())
// }


// #[test]
// fn test_prepare_invalid_sql_syntax() {
//     let conn = Connection::open_memory().expect("Failed to open Connection");
//     let sql = "CREAT TABLE name  (id INTEGER PRIMARY KEY);";  //typo for CREATE
    
//     let prepare_result = conn.prepare(sql);
//     assert!(prepare_result.is_err());
// }

// #[test]
// fn test_prepare_on_nonexistent_table() {
//     let conn = Connection::open_memory().expect("Failed to open Connection");
//     let sql = "SELECT * FROM nonexistent_table;";
    
//     let prepare_result = conn.prepare(sql);
//     assert!(prepare_result.is_err());
// }

// #[test]
// fn test_prepare_with_empty_string_is_ok() {
//     // TODO read on the note in Connection::prepare


//     // let conn = Connection::open_memory().expect("Test setup failed");
//     // let sql = "";
    
//     // let prepare_result = conn.prepare(sql);
//     // assert!(prepare_result.is_ok(), "Preparing an empty string should be OK and not error");
// }

