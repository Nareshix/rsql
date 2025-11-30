# SQL binding rules

*Note I might support more cases outside of this page. just planning only*

## General pattern
- bindings are often used in `WHERE` clause

- bindings often follow this pattern: expr <op\>
expr (bindings are ofen found on the RHS).

- Note: if a specifc synax **ONLY** allows column, that is not an expression.

- other common cases are `LIMIT` and `OFFSET` which are always int. `HAVING`  follows WHERE clause pattern. `BETWEEN` is a bit diff. Aggregate function with having, e.g. HAVING COUNT(*)> ?

- CTE and subquery are processed first. subquery are a bit harder as they break the left to right pattern, so focus on CTEs first

- arguments in funciton usually have fixed types. string concatenation is a special edge case ( | | )

## General guidelines
-  evaluate the type of LHS, and RHS follows that type (assuming there is binding parameters on RHS)

- whenever type is unable to be inferred, allow user to indicate the type. if too complex, make it runtime

- CAST(? AS INTEGER). explicit casting. good. Can be used when type cant be inferred. consider postgres :: style

- Dont forget about aliases. VERY IMPORTANT. aliases should be like "mapped"

## Implementation
1. 

## UPDATE
1. The type of the col on the LHS (e.g. id) must be equivalent to RHS
2.  **SET syntax isnt an expression.** Since, they only strictly allow col name, they are not an expression and needs to be handled seperately

```sql
UPDATE Customers
SET ContactName = ?, id = ?
WHERE CustomerID = ?;

```
- Note:  To find ?s, we need the type of LHS only. no need to evaluate RHS

```sql
UPDATE Customers
SET ContactName = ?, price = price + ?
WHERE price = ? + 1
```


### UPDATE FROM (Subquery. will focus later)


### LIMIT/OFFSET and ORDER BY
LIMIT OFFSET are always at the end of the query, so just check whether they exist **LAST** and the RHS is a binding parameter


## INSERT/UPSERT

### INSERT INTO table VALUES(...);
- for my usecase i will disallow implicit insertion (no columns specified). Easier to implement + safer as well

```sql
INSERT INTO Customers (name, Address, Country)
VALUES(?,?,?)
```
- each column name to each binding, sqlite will vverify this for us during prep stage so we cna just focus on type inference.


### INSERT INTO table SELECT (Subquery will focus later);

### INSERT INTO table DEFAULT VALUES (later);

### INSERT RETURNING (later todo)


## SELECT/DELETE
- Grouped them together cuz they are very similar.
- SELECT col_name op ? WHERE ..... col_name op ? is the expression and ? follows col_name
- All they have to do is literally evaluate expression (in `WHERE` or `HAVING` Clause)


## for delete, u can use limitoffset and order by

# Misc
- existence check may not necesarily follow the LHS RHS rule cuz they always evalute to bool. For instance

  ```sql
   SELECT * FROM users WHERE name_str IN (1 + ?, 'hi', 1.23)
  ```
  werid idt anyone would do it, but in this case ? would be an int/REAL



- Techincally IN (?,?,?) should allow multiple types, but for simplicty we wil make it all same. if i want to ever have diff types, then explicitly cast it

-  consider CASE Expressions

- ? is Bool in this case
```sql
 CASE WHEN ? THEN ...
```

- ? is real or int
```sql
CASE WHEN ? > 10 THEN ...
```
some ai notes:

THEN/ELSE clause: These determine the return type of the CASE expression.
If the CASE is being compared to a column (WHERE status = CASE ... END), the THEN/ELSE bindings take the type of that column.
CASE WHEN x THEN ? ELSE ? END: All result branches should share the same type.


---
*Random shiet*

1. nearly all select statemnets start with Query. It is only in Body where u need to parse the Select() enum Which subsequently contains Select struct

2. In Select struct, these are the only things u need to focus.
   - projection
   - from
   - selection (basically ur WHERE.)
   - group_by
   - having
   - distinct
   - (note order by and limit is in above Query)


for binding parameter, general pattern

1. expr, op, binding_parameter [x]
2. functions
3. Between clause [x]
4. INSERT INTO users (name, email) VALUES (?, ?);
5. Case (nth special but keep in mind)
6. LIMIT and OFFSET [x]
7. Update set [x]
8. DELETE WHERE [x]
9. LIKE [x]
10. HAVING

INSERT statements: INSERT INTO users VALUES (?, ?, ?)
UPDATE statements: UPDATE users SET name = ? WHERE id = ?
CASE expressions: CASE WHEN ? THEN value END
Function arguments: SUBSTR(name, ?, ?)
IS NULL/IS NOT NULL with placeholders
Unary operations: -?, NOT ?
Nested queries/subqueries with placeholders
Common Table Expressions (CTEs)