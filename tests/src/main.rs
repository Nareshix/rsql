use rsql::{Connection, LazyStmt, lazy_sql};
use std::ffi::CString;
use std::ptr;
use libsqlite3_sys::sqlite3_exec;

// 1. FIX: Define valid SQL. 
// Cannot be empty, otherwise stmt pointer is NULL -> Segfault.
#[lazy_sql]
pub struct UserDao<'a> {
    db: &'a Connection,

    // Using "INSERT OR REPLACE" so we can run this code multiple times without unique constraint errors
    #[sql("INSERT OR REPLACE INTO users (id, name) VALUES (?, ?)")]
    insert_stmt: LazyStmt,

    #[sql("SELECT * FROM users WHERE id = ?")]
    get_by_id_stmt: LazyStmt,
}

fn main() {
    let conn = Connection::open("test.db").unwrap();

    let create_table_sql = CString::new(
        "CREATE TABLE IF NOT EXISTS users (id INTEGER PRIMARY KEY, name TEXT)"
    ).unwrap();
    
    unsafe {
        let rc = sqlite3_exec(conn.db, create_table_sql.as_ptr(), None, ptr::null_mut(), ptr::null_mut());
        if rc != 0 {
            panic!("Failed to create table: {}", rc);
        }
    }
    println!("2. Table 'users' ensured to exist.");

    // 3. Create DAO
    let mut dao = UserDao::new(&conn);
    println!("3. DAO created (Statements are lazy/NULL).");

    // --- WRITE Phase (Insert) ---
    {
        println!("4. Preparing INSERT statement...");
        // This calls your macro logic -> prepares stmt -> stores in dao
        let mut stmt = dao.insert_stmt().expect("Failed to prepare insert");

        // Bind ID (1) and Name ("Mom")
        // Assuming your ToSql trait handles i32 and &str
        stmt.bind_parameter(1, 1).expect("Failed to bind ID");
        stmt.bind_parameter(2, 1.23).expect("Failed to bind Name");

        println!("5. Executing INSERT...");
        
        // This calls your PreparredStmt::step() logic
        match stmt.step() {
            Ok(_) => println!("   -> Success! User 'Mom' (id: 1) inserted."),
            Err(e) => println!("   -> Error inserting: {:?}", e), // Debug print if your error supports it
        }

    } // stmt goes out of scope here.
      // Your PreparredStmt::drop runs: calls sqlite3_reset & clear_bindings.
      // The RAW pointer inside 'dao' stays valid.

    println!("6. Scope ended. Statement reset/cached.");

    // --- CACHE CHECK ---
    {
        println!("7. Requesting INSERT statement again...");
        // This should NOT call sqlite3_prepare_v2 again (Fast path)
        let mut stmt_again = dao.insert_stmt().expect("Failed to get cached stmt");
        
        stmt_again.bind_parameter(1, 2).unwrap();
        stmt_again.bind_parameter(2, "Dad").unwrap();
        
        stmt_again.step().expect("Failed to insert Dad");
        println!("   -> Success! User 'Dad' (id: 2) inserted.");
    }

    println!("--- Happy Path Complete. Check 'mom.db' ---");

} // dao drops -> LazyStmt drops -> sqlite3_finalize (Safe cleanup)
  // conn drops -> sqlite3_close