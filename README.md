
## Quick Start

```rust
use rsql::{lazy_sql, LazyConnection};

#[lazy_sql]
struct AppDatabase {
    // all create tables must be at the top before read/write logic in order to get compile time cheks

    // dont have to import sql! macro. lazy_sql brings along with it
    init: sql!("
        CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY NOT NULL,
            username TEXT NOT NULL,
            is_active INTEGER NOT NULL CHECK (is_active IN (0, 1)) -- the library infers this as bool. more info below
        )
    "),

    add_user: sql!("INSERT INTO users (username, is_active) VALUES (?, ?)"),

    get_active_users: sql!("SELECT id as user_id, username FROM users WHERE is_active = ?"),
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let conn = LazyConnection::open_memory()?;

    // The 'new' constructor is generated automatically
    let mut db = AppDatabase::new(&conn);

    // You can now call the methods and it will run the sql commands
    db.init()?;

    // Types are enforced by Rust
    db.add_user("Alice", true)?;
    db.add_user("Bob", false)?;

    // active_users is an iterator
    let active_users = db.get_active_users(true)?;

    for user in active_users {
        // u can access the fields specifically if you want
        // Respects Aliases (id -> user_id)
        println!("{} {}", user.username, user.user_id);
    }

    Ok(())
}
```

Since SQLite defaults to nullable columns, the type inference system defaults to Option<T>. To use concrete types (e.g., String instead of Option<String>), explicitly add NOT NULL to your table definitions

It is strongly recommended to use [STRICT tables](https://sqlite.org/stricttables.html) for better compile time guarantees

The type inference system and compile time check also works well for `joins`, `ctes`, `window function`, `recursive ctes` and more complex scenarios.



## Configuration Methods

`rsql` supports 3 ways to define your schema, depending on your workflow.

### 1. Inline Schema (Standalone)
As seen in the Quick Start. Define tables inside the struct. Useful for prototypes or self-contained modules.
```rust
#[lazy_sql]
struct App { ... }
```

### 2. SQL File
Point to a `.sql` file. The compile time checks will be done against this sql file (ensure that there is `CREATE TABLE`)   `rsql` watches this file; if you edit it, your Rust project recompiles automatically to ensure type safety.
```rust
#[lazy_sql("schema.sql")]
// you dont have to create tables and go on writing read/write sql queries and get compile time guarantees.
struct App { ... }
```

### 3. Live Database
Point to an existing `.db` binary file. `rsql` inspects the live metadata to validate your queries.
```rust
#[lazy_sql("production_snapshot.db")]
struct App { ... }
```

Note: for method 2 and 3, you can techinically CREATE TABLE as well but to ensure that they are taken into considreration for compile time check, add them at the top of your struct



## Features & Usage
the `lazy_sql!` macro brings along `sql!` and `sql_runtime!` macro. so there is no need to import them. and they can only be used within structs defined with `lazy_sql!`
### The `sql!` Macro
Always prefer to use this. It automatically:
1.  **Infers Inputs:** Maps `?` to Rust types (`i64`, `f64`, `String`, `bool`).
2.  **Generates Outputs:** For `SELECT` queries, creates a struct named after the field

### The `sql_runtime!` Macro
Use this only when u need the sql to to be runned at runtime. And there are some additional things to take note of when using this macro

#### 1. Mapping to Custom Structs (`SqlMapping`)
You can map a query result to any struct by deriving `SqlMapping`.

 `SqlMapping` maps columns by **index**, not by name. The order of fields in your struct **must** match the order of columns in your `SELECT` statement exactly.

```rust
use rsql::{SqlMapping, LazyConnection, lazy_sql};

#[derive(Debug, SqlMapping)]
struct UserStats {
    total: i64,      // Maps to column index 0
    status: String,  // Maps to column index 1
}

#[lazy_sql]
struct Analytics {
    get_stats: sql_runtime!(
        UserStats, // pass i the struct so u can access the fields later
        "SELECT count(*) as total, status
         FROM users
         WHERE id > ? AND login_count >= ?
         GROUP BY status",
        i64, // Maps to 1st '?'
        i64  // Maps to 2nd '?'
    )
}

fn foo{
    let conn = LazyConnection::open_memory()?;
    let mut db = Analytics::new(&conn);

    let foo = db.get_stats(100, 5)?;
    for i in foo{
        // i.total and i.status is accessible
    }
}
```


#### 2. No Return Type (Void)
For `INSERT`, `UPDATE`, or `DELETE` statements

```rust
#[lazy_sql]
struct Logger {
    log: sql_runtime!("INSERT INTO logs (msg, level) VALUES (?, ?)", String, i64)
}
// can continue to use it normally.
```

### Postgres Compatibility
You can use Postgres-style casting syntax. `rsql` transpiles it to SQLite syntax at compile time.
```rust
// You write:
sql!("SELECT price::text FROM items")

// Compiles to:
// "SELECT CAST(price AS TEXT) FROM items"
```

### Type Mapping
| SQLite Context | Rust Type | Notes |
| :--- | :--- | :--- |
| `TEXT` | `String` / `&str` | |
| `INTEGER` | `i64` | |
| `REAL` | `f64` | Includes `FLOAT`, `DOUBLE` |
| `BOOLEAN` | `bool` | Requires `CHECK (col IN (0,1))`  or `Check (col = 0 OR col = 1)`. You could techinically use `BOOL` or `BOOLEAN` as the data type when creating table (due to sqlite felxible type nature) and it would work as well. But this is discouraged |
| Nullable | `Option<T>` | When a column or expr has a possibility of returning `NULL`, this will be returned. its recommended to use `NOT NULL` when creating tables so that ergonoimic wise you dont have always use Some(T) when adding parameters |

### Strict INSERT Validation
`rsql` checks `INSERT` statements at compile time. If you omit a column that is **not null** and has **no default value**, your code will fail to compile.