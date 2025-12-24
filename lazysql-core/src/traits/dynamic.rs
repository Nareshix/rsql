#[derive(Debug, Clone)]
pub enum Value {
    Integer(i64),
    Real(f64),
    Text(String),
    // Blob(Vec<u8>),
    Null,
}

impl Value {
    pub fn as_string(&self) -> String {
        match self {
            Value::Integer(i) => i.to_string(),
            Value::Real(f) => f.to_string(),
            Value::Text(s) => s.clone(),
            // Value::Blob(_) => "<Binary Data>".to_string(),
            Value::Null => "NULL".to_string(),
        }
    }
}