pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

mod expr;
mod select_result_type;
mod table;

#[cfg(test)]
mod tests {
    use crate::expr::{BaseType, Type};
    use crate::table::{ColumnInfo, create_tables};
    use std::collections::HashMap;

    // TODO simple test only not comprehensive do more creation of table
    #[test]
    fn test_create_tables_no_check_constraint() {
        let sql = "CREATE TABLE users (id INTEGER, name TEXT, wow REAL); ";
        let mut tables = HashMap::new();
        create_tables(sql, &mut tables);

        let mut expected = HashMap::new();
        let columns = vec![
            ColumnInfo {
                name: "id".to_string(),
                data_type: Type { base_type: BaseType::Integer, nullable: false },
                check_constraint: None,
            },
            ColumnInfo {
                name: "name".to_string(),
                data_type: Type { base_type: BaseType::Text, nullable: false },
                check_constraint: None,
            },
            ColumnInfo {
                name: "wow".to_string(),
                data_type: Type { base_type: BaseType::Real, nullable: false },
                check_constraint: None,
            },
        ];

        expected.insert("users".to_string(), columns);

        assert_eq!(tables, expected);
    }
}
