pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

mod select_result_type;
mod expr;
mod table;

#[cfg(test)]
mod tests {
    use crate::expr::Type;
    use crate::table::{FieldInfo, create_table};
    use std::collections::HashMap;

    // TODO simple test only not comprehensive do more creation of table
    #[test]
    fn test_create_table_no_check_constraint() {
        let sql = "CREATE TABLE users (id INTEGER, name TEXT, wow REAL); ";
        let mut tables = HashMap::new();
        create_table(sql, &mut tables);

        let mut expected = HashMap::new();
        let fields = vec![
            FieldInfo {
                name: "id".to_string(),
                data_type: Type::Int,
                check_constraint: None,
            },
            FieldInfo {
                name: "name".to_string(),
                data_type: Type::String,
                check_constraint: None,
            },
            FieldInfo {
                name: "wow".to_string(),
                data_type: Type::Float,
                check_constraint: None,
            },
        ];

        expected.insert("users".to_string(), fields);

        assert_eq!(tables, expected);
    }
}
