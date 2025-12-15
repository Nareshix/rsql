use rsql::{internal_sqlite::efficient::lazy_connection::LazyConnection, lazy_sql};

#[lazy_sql]
pub struct ShopDao {
    create_table: sql!(
        " CREATE TABLE Persons (
        PersonID INTEGER NOT NULL,
        LastName TEXT NOT NULL,
        FirstName TEXT,
        Address TEXT,
        Alive INTEGER NOT NULL CHECK (Alive IN (0,1))
    ) STRICT;"
    ),
    /// comment issue
    insert: sql!(
        "INSERT INTO Persons (PersonID, LastName, FirstName, Address, Alive)
        VALUES (1, 'Smith', 'John', '123 Main Street', ?);"
    ),

    /// your mom
    select: sql!("SELECT * FROM persons"),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = LazyConnection::open_memory().unwrap();
    let mut dao = ShopDao::new(&conn);

    dao.create_table()?;
    dao.insert(true)?;

    let results = dao.select()?;
    for i in results {
        println!("{:?}", i?.alive);
    }
    Ok(())
}
