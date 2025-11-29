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
7. Update set
8. DELETE WHERE
9. LIKE [x]
10. HAVING