use rsql::{Connection, LazyStmt, lazy_sql, utility::utils::prepare_stmt};

#[lazy_sql]
pub struct UserDao<'a> {
    db: &'a Connection,

    #[sql("SELECT * FROM users WHERE id = ?")]
    get_by_id_stmt: LazyStmt,

    #[sql("")]
    insert_stmt: LazyStmt,
}

fn main() {
    let conn = Connection::open_memory().unwrap();

    let mut dao = UserDao::new(&conn);
    let stmt = dao.get_by_id_stmt();
    println!("Dao created!");
}
