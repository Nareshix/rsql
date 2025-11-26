-- I will mainly focus on the pracical examples and common sql queryies that i use.
-- trivial things like select ?, WHERE ? = ? would be given lower prio

1. SELECT * FROM user 

-- if im querying from one table, dont bother making it "explicit" in this case
2. SELECT name FROM user

-- when querying from more than 1 table, then make it explicit 
3. SELECT user.name, class.id FROM user, class

-- any binary operators (+,-,*,/,%,<,>,>=,<=, <>, etc.)
4. SELECT salary * 12 FROM user

-- when given an alias, that alias is wht i will have access to in rust
5. SELECT salary * 12 AS Annual_Salary FROM user
 

-- Most Aggregate functions
6. SELECT SUM(salary) FROM user


-- JOINS. Inner join is awlays valid dataype depending on whts being merged.
-- Left join has possibilty of RHS  being NULL so Option<T> for that
7. 

-- CASES gotta be supported
8. 

-- wildcard and specific table alias is quite useful 
9.SELECT employees.* -- This grabs ALL columns, but ONLY from the employees table
    FROM employees
    INNER JOIN departments 
    ON employees.DeptID = departments.ID

-- CAST also. postgres shorthand of :: seems interesting
10. SELECT 'ORD-' + CAST(1001 AS TEXT);

-- String concatenation (not the function but the | |)
11. 


--sql functions. Theres a lot, but ill just focus on the ones i like. most of them dont have to be done
-- in db and can be done in codebase
-- lower/upper, trim, length, replace, 
-- coalesce(X, Y,...)
-- iif(B1, V1, V2) shorthand for CASE :O
-- 
12. 


-- the usuals 
-- avg(X) 
-- count(*) 
-- count(X) 
-- max(X) 
-- min(X) 
-- sum(X) 
13.

-- MaTh. Interestlingly, all math functions returns a float. The input can be either 
-- int, float, text (coercion but we will prevent this). Note the Null issue also applies here
14.


-- datetime is small so might as well cover all eh
-- ALl return TEXT . exceptions are indicated
-- date 
-- time
-- datetime
-- julianday - REAL
-- unixepoch - INTEGER
-- strftime
-- timediff
15. 


-- CTEs are interesting
16.

-- Window funcitons seems complicaed. ;-;
17.


-- some ppl love QUALIFY sql ubt sqlite dont support it
18.

19. bool logic
SELECT id,
    (salary > 100000) AS is_high_earner, -- Returns Boolean (bool)
    active IS NOT NULL AS is_active -- Returns Boolean (bool)
FROM user

-- json??
20. 


-- well again gonna focus on the common ones and ignore the weird impractical edge cases

1. SELECT * FROM users WHERE email = ?;

2. INSERT INTO orders (user_id, product_name, price) VALUES (?, ?, ?);

3. UPDATE inventory
    SET quantity = ?,
    last_updated = ?
    WHERE product_id = ?;

4. SELECT *
FROM posts
WHERE title LIKE ?

5. SELECT *
FROM posts
WHERE title LIKE CONCAT('%', ?, '%')


6. SELECT *
FROM employees
WHERE id IN (?, ?, ?);

7.SELECT *
FROM logs
ORDER BY created_at DESC
LIMIT ? OFFSET ?;



8. SELECT * FROM transactions WHERE amount BETWEEN ? AND ?;
-- OR
9. SELECT * FROM users WHERE age > ?;


10. SELECT category,
    COUNT(*) as total
FROM products
GROUP BY category
HAVING count(*) > ?;
-- Only show categories with more than X products


11. SELECT order_id,
    CASE
        WHEN amount > ? THEN 'High Value' -- Bind the threshold for "High"
        ELSE 'Standard'
    END as label
FROM orders;


12. SELECT price * ? as price_with_tax
FROM products;    
INSERT INTO settings (user_id, theme) VALUES (?, ?)
ON DUPLICATE KEY UPDATE theme = ?; 
-- Note: You often have to bind the value twice (once for insert, once for update)

  

-- or UPSERT
 13.  INSERT INTO settings (user_id, theme)
VALUES (?, ?) ON DUPLICATE KEY
UPDATE theme = ?;
-- Note: You often have to bind the value twice (once for insert, once for update)


-- other builit in functions as well (look above)


-- joins
SELECT *
FROM users u
    LEFT JOIN orders o ON u.id = o.user_id
    AND o.status = ? -- Only join orders if they have this status


SELECT id,
    email,
    ? as source_tag -- We inject a static string here (e.g., 'batch_run_1')
FROM users;


WITH RecentSales AS (
    SELECT *
    FROM sales
    WHERE sale_date > ?
)
SELECT *
FROM RecentSales;