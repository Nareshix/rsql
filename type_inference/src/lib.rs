pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

mod bindings;
mod expr;
mod table;

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use crate::{expr::{Type, get_type_of_columns_from_select}, table::{FieldInfo, create_table}}; 

    fn setup() -> HashMap<String, Vec<FieldInfo>> {
        let mut tables = HashMap::new();
        create_table("CREATE TABLE users (id INTEGER, name TEXT, is_active INTEGER);", &mut tables);
        create_table("CREATE TABLE orders (order_id INTEGER, user_id INTEGER, amount REAL);", &mut tables);
        tables
    }

    #[test]
    fn test_simple_select() {
        let tables = setup();
        let sql = "SELECT id, name FROM users";
        let types = get_type_of_columns_from_select(sql, &tables);
        
        assert_eq!(types, vec![Type::Int, Type::String]);
    }

    #[test]
    fn test_compound_identifiers() {
        // Tests explicitly selecting table.column
        let tables = setup();
        let sql = "SELECT users.id, orders.amount FROM users, orders";
        let types = get_type_of_columns_from_select(sql, &tables);
        
        assert_eq!(types, vec![Type::Int, Type::Float]);
    }

    #[test]
    fn test_implicit_identifiers_in_join() {
        // Tests inferring column source from multiple tables
        let tables = setup();
        let sql = "SELECT name, amount FROM users INNER JOIN orders ON users.id = orders.user_id";
        let types = get_type_of_columns_from_select(sql, &tables);
        
        assert_eq!(types, vec![Type::String, Type::Float]);
    }

    #[test]
    // TODO bool not implemented
    fn test_wildcard() {
        let tables = setup();
        let sql = "SELECT * FROM users";
        let types = get_type_of_columns_from_select(sql, &tables);
        
        // users: id, name, is_active
        assert_eq!(types, vec![Type::Int, Type::String, Type::Int]);
    }

    #[test]
    fn test_qualified_wildcard() {
        let tables = setup();
        // Should only return columns from 'orders'
        let sql = "SELECT orders.* FROM users, orders";
        let types = get_type_of_columns_from_select(sql, &tables);
        
        // orders: order_id, user_id, amount
        assert_eq!(types, vec![Type::Int, Type::Int, Type::Float]);
    }

            // Expr::Value(val) =>{
            
            // // identifies whether its a float or int
            // let numeral = &val.value;
            //  match numeral {
            //     Value::Number(num, _) => {
            //         // TODO scientific notation


    #[test]
    fn test_literals() {
        let tables = setup();
        // String, Int, Float, Bool, Null
        let sql = "SELECT 'hello', 123, 45.6+1, TRUE, NULL FROM users";
        let types = get_type_of_columns_from_select(sql, &tables);
        
        assert_eq!(types, vec![
            Type::String, 
            Type::Int, 
            Type::Float, 
            Type::Bool, 
            Type::Null
        ]);
    }

    #[test]
    fn test_math_operations() {
        let tables = setup();
        // Int + Int = Int
        // Int + Float = Float
        let sql = "SELECT 10 + 5, 10 + 5.5 FROM users";
        let types = get_type_of_columns_from_select(sql, &tables);
        
        assert_eq!(types, vec![Type::Int, Type::Float]);
    }

    #[test]
    fn test_comparison_ops() {
        let tables = setup();
        // Comparisons always return Bool
        let sql = "SELECT amount > 100, id = 5 FROM orders";
        let types = get_type_of_columns_from_select(sql, &tables);
        
        assert_eq!(types, vec![Type::Bool, Type::Bool]);
    }

    #[test]
    fn test_alias() {
        let tables = setup();
        let sql = "SELECT name AS full_name, amount AS total_price FROM users, orders";
        let types = get_type_of_columns_from_select(sql, &tables);
        
        assert_eq!(types, vec![Type::String, Type::Float]);
    }

    #[test]
    fn test_column_not_in_scope() {
        let tables = setup();
        // 'amount' exists in orders, but we are only selecting from 'users'
        // This simulates a logical error in the SQL, returning Unknown for that column
        let sql = "SELECT amount FROM users";
        let types = get_type_of_columns_from_select(sql, &tables);
        
        assert_eq!(types, vec![Type::Unknown]);
    }

    #[test]
    fn test_full_outer_join() {
        let tables = setup();
        let sql = "SELECT name, amount FROM users FULL OUTER JOIN orders ON users.id = orders.user_id";
        let types = get_type_of_columns_from_select(sql, &tables);

        assert_eq!(types, vec![Type::String, Type::Float]);
    }

}