fn pg_to_standard(sql: &str) -> String {
    let mut chars: Vec<char> = sql.chars().collect();
    let mut i = 0;

    // We store indices of '::' here
    let mut cast_indices = Vec::new();

    // STATE MACHINE VARIABLES
    let mut in_quote = false;      // Are we inside ' or " ?
    let mut quote_char = '\0';     // Which quote started it?
    let mut in_comment = false;    // Are we inside a -- comment?

    // 1. SCAN PASS: Find the '::' operators safely
    while i < chars.len() {
        let c = chars[i];
        let next_c = if i + 1 < chars.len() { chars[i+1] } else { '\0' };

        if in_comment {
            if c == '\n' { in_comment = false; }
        }
        else if in_quote {
            if c == quote_char {
                // Check for escaped quote (e.g. 'Don''t')
                if next_c == quote_char {
                    i += 1; // Skip the escape
                } else {
                    in_quote = false;
                }
            }
        }
        else {
            // Not in quote, Not in comment
            if c == '-' && next_c == '-' {
                in_comment = true;
                i += 1;
            } else if c == '\'' || c == '"' {
                in_quote = true;
                quote_char = c;
            } else if c == ':' && next_c == ':' {
                // FOUND IT!
                cast_indices.push(i);
                i += 1; // Skip second colon
            }
        }
        i += 1;
    }

    // 2. REPLACEMENT PASS: Work Right-to-Left
    for &idx in cast_indices.iter().rev() {

        // --- FIND RIGHT HAND SIDE (The Type) ---
        // e.g. "int", "varchar(20)", "decimal(10, 2)"
        let mut rhs_end = idx + 2;

        // Skip leading spaces
        while rhs_end < chars.len() && chars[rhs_end].is_whitespace() { rhs_end += 1; }

        let mut p_depth = 0;
        while rhs_end < chars.len() {
            let c = chars[rhs_end];
            // Stop at separators if we aren't inside type parentheses (like varchar(x))
            if p_depth == 0 && (c == ' ' || c == ',' || c == ')' || c == ';' || c == '\n') {
                break;
            }
            if c == '(' { p_depth += 1; }
            if c == ')' { p_depth -= 1; }
            rhs_end += 1;
        }

        // --- FIND LEFT HAND SIDE (The Value) ---
        // e.g. "col", "'val'", "(a + b)"
        let mut lhs_start = idx;

        // Skip trailing spaces backwards
        while lhs_start > 0 && chars[lhs_start - 1].is_whitespace() { lhs_start -= 1; }

        if lhs_start > 0 {
            let end_char = chars[lhs_start - 1];

            if end_char == ')' {
                // Case A: Grouped Expression "(a + b)::int"
                // Walk backwards balancing parens
                let mut balance = 1;
                lhs_start -= 1;
                while lhs_start > 0 && balance > 0 {
                    lhs_start -= 1;
                    if chars[lhs_start] == ')' { balance += 1; }
                    if chars[lhs_start] == '(' { balance -= 1; }
                }
            } else if end_char == '\'' {
                // Case B: String Literal "'2023'::text"
                lhs_start -= 1;
                while lhs_start > 0 {
                     lhs_start -= 1;
                     // Stop at next quote if not escaped
                     if chars[lhs_start] == '\'' && (lhs_start == 0 || chars[lhs_start-1] != '\'') {
                         break;
                     }
                }
            } else {
                // Case C: Simple Identifier "table.col::int"
                // Walk back until we hit a math operator, space, or comma
                while lhs_start > 0 {
                    let c = chars[lhs_start - 1];
                    // Valid identifier chars: Letters, Numbers, _, ., "
                    if !c.is_alphanumeric() && c != '_' && c != '.' && c != '"' {
                        break;
                    }
                    lhs_start -= 1;
                }
            }
        }

        // DO THE SWAP
        let val: String = chars[lhs_start..idx].iter().collect();
        let type_name: String = chars[(idx+2)..rhs_end].iter().collect();
        let new_str = format!("CAST({} AS {})", val.trim(), type_name.trim());

        chars.splice(lhs_start..rhs_end, new_str.chars());
    }

    chars.into_iter().collect()
}

fn main() {
    let sql = r#"
        SELECT
            '123'::int,
            (price * 1.5)::decimal(10,2),
            -- This comment shouldn't break::things
            table."col name"::text,
            ((nested))::int
        FROM users
    "#;

    println!("{}", pg_to_standard(sql));
}