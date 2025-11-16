#[derive(Debug, rsql::SqlMapping)]
#[allow(unused)]
struct Person {
    url: String,
    caption: String,
}

use std::time::Instant;

// results for select * from table
// rsql Elapsed: 136.88s
// rusqlite Elapsed: 137.13s
fn main() -> Result<(), Box<dyn std::error::Error>> {
    // rsql
    let conn = rsql::Connection::open("wukong.db").unwrap();

    let now = Instant::now();

    let statement = conn.prepare("SELECT * FROM wukong_data WHERE url LIKE '%gss0%'")?;
    #[allow(unused)]
    for person in statement.query(Person) {
        // println!("{:?}", person);
    }

    let elapsed_rsql = now.elapsed();
    // endrsql

    //rusqlite
    let now = Instant::now();

    let conn = rusqlite::Connection::open("wukong.db")?;
    let mut stmt = conn.prepare_cached("SELECT * FROM wukong_data WHERE url LIKE '%gss0%'")?;
    let person_iter = stmt.query_map([], |row| {
        Ok(Person {
            url: row.get(0)?,
            caption: row.get(1)?,
        })
    })?;

    #[allow(unused)]
    for person in person_iter {
        // println!("{:?}", person.unwrap())
    }

    let elapsed_rusqlite = now.elapsed();
    //endrusqlite

    println!("rsql Elapsed: {:.2?}", elapsed_rsql);
    println!("rusqlite Elapsed: {:.2?}", elapsed_rusqlite);

    Ok(())
}
