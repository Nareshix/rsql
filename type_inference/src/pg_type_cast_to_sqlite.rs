pub fn pg_cast_syntax_to_sqlite(sql: &str) -> String {
    let mut chars: Vec<char> = sql.chars().collect();
    let mut i = 0;

    let mut cast_indices = Vec::new();

    let mut in_quote = false;
    let mut quote_char = '\0';
    let mut in_comment = false;

    while i < chars.len() {
        let c = chars[i];
        let next_c = if i + 1 < chars.len() {
            chars[i + 1]
        } else {
            '\0'
        };

        if in_comment {
            if c == '\n' {
                in_comment = false;
            }
        } else if in_quote {
            if c == quote_char {
                if next_c == quote_char {
                    i += 1;
                } else {
                    in_quote = false;
                }
            }
        } else if c == '-' && next_c == '-' {
            in_comment = true;
            i += 1;
        } else if c == '\'' || c == '"' {
            in_quote = true;
            quote_char = c;
        } else if c == ':' && next_c == ':' {
            cast_indices.push(i);
            i += 1;
        }
        i += 1;
    }

    for &idx in cast_indices.iter().rev() {
        let mut rhs_end = idx + 2;

        while rhs_end < chars.len() && chars[rhs_end].is_whitespace() {
            rhs_end += 1;
        }

        let mut p_depth = 0;
        while rhs_end < chars.len() {
            let c = chars[rhs_end];

            if p_depth == 0 {
                if c.is_whitespace() {
                    break;
                }
                if ",);".contains(c) {
                    break;
                }
                if "+-*/=<>!^%|~".contains(c) {
                    break;
                }
            }

            if c == '(' {
                p_depth += 1;
            }
            if c == ')' {
                p_depth -= 1;
            }
            rhs_end += 1;
        }

        let mut lhs_start = idx;

        // Skip initial spaces
        while lhs_start > 0 && chars[lhs_start - 1].is_whitespace() {
            lhs_start -= 1;
        }

        if lhs_start > 0 {
            let end_char = chars[lhs_start - 1];

            if end_char == ')' {
                // Balance parenthesis backwards
                let mut balance = 1;
                lhs_start -= 1;
                while lhs_start > 0 && balance > 0 {
                    lhs_start -= 1;
                    if chars[lhs_start] == ')' {
                        balance += 1;
                    }
                    if chars[lhs_start] == '(' {
                        balance -= 1;
                    }
                }
            } else if end_char == '\'' || end_char == '"' {
                // Handle quoted strings/identifiers backwards
                let q = end_char;
                lhs_start -= 1;
                while lhs_start > 0 {
                    lhs_start -= 1;
                    if chars[lhs_start] == q {
                        // Check for escaped quote (e.g. 'Don''t')
                        if lhs_start > 0 && chars[lhs_start - 1] == q {
                            lhs_start -= 1;
                        } else {
                            break;
                        }
                    }
                }
            } else {
                while lhs_start > 0 {
                    let c = chars[lhs_start - 1];

                    if c.is_whitespace() {
                        break;
                    }
                    if ",();".contains(c) {
                        break;
                    }
                    if "+-*/=<>!^%|~".contains(c) {
                        break;
                    }

                    lhs_start -= 1;
                }
            }
        }

        let val: String = chars[lhs_start..idx].iter().collect();
        let type_name: String = chars[(idx + 2)..rhs_end].iter().collect();
        let new_str = format!("CAST({} AS {})", val.trim(), type_name.trim());

        chars.splice(lhs_start..rhs_end, new_str.chars());
    }

    chars.into_iter().collect()
}