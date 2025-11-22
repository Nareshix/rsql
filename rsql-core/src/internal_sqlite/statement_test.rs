// use super::connection::Connection;


// //TODO


// #[test]
// fn test_single_binding() {
//     let conn = Connection::open("asd.db").expect("Failed to open Connection");
//     let stmt = conn
//         .prepare("SELECT * FROM test WHERE name = $1 AND id= $2")
//         .expect("failed to create Statement object");

//     stmt.bind_parameter(1, "Alice").expect("failed to bind parameters. This might happen due to not having the asd.db file in root directory of rsql"); 

//     let _ = stmt.step();

// }


// #[test]
// fn test_multiple_binding() {
//     let conn = Connection::open("asd.db").expect("Failed to open Connection");
//     let stmt = conn
//         .prepare("SELECT * FROM test WHERE name = $1 AND id= $2")
//         .expect("failed to create Statement object");

//     stmt.bind_parameter(1, "Alice").expect("failed to bind parameters. This might happen due to not having the asd.db file in root directory of rsql"); 
//     stmt.bind_parameter(2, "Alice").expect("failed to bind parameters. This might happen due to not having the asd.db file in root directory of rsql"); 

//     let _ = stmt.step();

// }