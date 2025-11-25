1. SELECT t2.id 
FROM t1 
LEFT JOIN t2 ON t1.id = t2.id

t2.id can be either null or a type (cuz its left join not inner join)

2. SELECT col_a + col_b FROM table
only works if they are int, or floatr and they must be not null

3. SELECT CASE WHEN x > 10 THEN 1 ENDk
There is no ELSE. SQL implies ELSE NULL. Therefore, the result type is Nullable INT, even if the THEN branch returns a hard constant.

4. SELECT 10 / 3

5. SELECT decimal_col_a * decimal_col_b

6. SELECT CASE WHEN x THEN 'Hello' ELSE 123 END
Strong typing: Should throw a Type Mismatch Error. or an OR type

7. SELECT CASE WHEN 1=0 THEN 1 ELSE NULL END

8. SELECT * FROM users WHERE id IN (?)

9. SELECT col_a || col_b

10. aggregate functions

11. recursion  mayeb not support
12. will supporccte and window funciotn eventually


-- 1.1. Int/Int Math -> Returns INTEGER
SELECT id + 10 FROM users;
SELECT id - 10 FROM users;
SELECT id * 10 FROM users;
SELECT id / 10 FROM users; -- SQLite integer division behaves like C (returns INT)

-- 1.2. Real/Int Math -> Returns REAL
SELECT rating + 10 FROM users;
SELECT rating * id FROM users;

-- 1.3. Modulo -> Returns INTEGER
SELECT id % 2 FROM users;

-- 1.4. Parameter Inference -> ? must imply INTEGER
SELECT id + ? FROM users;


-- 2.1. Concatenation -> Returns TEXT
SELECT username || ' is cool' FROM users;

-- 2.2. Mixed Concat (Strict Mode) -> Should ERROR if id is not cast to string
SELECT username || id FROM users; 

-- 2.3. Parameter Inference -> ? must imply TEXT
SELECT username || ? FROM users;
-- 3.1. Comparison -> Returns BOOLEAN
SELECT id > 10 FROM users;
SELECT rating <= 5.5 FROM users;
SELECT username = 'admin' FROM users;
SELECT username != 'guest' FROM users;

-- 3.2. Logical AND/OR -> Returns BOOLEAN
SELECT (id > 5) AND (score < 100) FROM users;

-- 3.3. IS NULL Checks -> Returns BOOLEAN (Always NOT NULL)
SELECT score IS NULL FROM users;
SELECT score IS NOT NULL FROM users;

-- 3.4. Parameter Inference -> ? must imply type of left-hand column
SELECT * FROM users WHERE username = ?; -- ? is TEXT
SELECT * FROM users WHERE score > ?;    -- ? is INTEGER
****
-- 4.1. NotNull + Nullable -> Returns Nullable INT
SELECT id + score FROM users;

-- 4.2. Nullable + Nullable -> Returns Nullable INT
SELECT score + score FROM users;

-- 4.3. Force Not Null (COALESCE) -> Returns INT (Not Null)
SELECT COALESCE(score, 0) FROM users;

-- 4.4. Parameter Inference with Coalesce
-- ? implies INTEGER because 'score' is INTEGER
SELECT COALESCE(score, ?) FROM users;


-- 5.1. CASE with ELSE -> Returns type of branches (TEXT)
SELECT 
  CASE 
    WHEN score > 100 THEN 'High' 
    ELSE 'Low' 
  END 
FROM users;

-- 5.2. CASE without ELSE -> Returns TEXT | NULL
SELECT 
  CASE 
    WHEN score > 100 THEN 'High' 
  END 
FROM users;

-- 5.3. IIF (SQLite shortcut) -> Returns REAL | NULL
SELECT IIF(id > 5, 10.5, NULL) FROM users;


Logic: SUM/AVG can return NULL even if input is NOT NULL (on empty set).

-- 6.1. COUNT -> Always Returns INTEGER (Not Null)
SELECT COUNT(id) FROM users;
SELECT COUNT(*) FROM users;

-- 6.2. SUM (Int) -> Returns INTEGER | NULL
SELECT SUM(id) FROM users;

-- 6.3. AVG -> Always Returns REAL | NULL
SELECT AVG(id) FROM users;

-- 6.4. MIN/MAX -> Returns Type of Column | NULL
SELECT MAX(username) FROM users; -- Returns TEXT | NULL


-- 7.1. LIMIT/OFFSET -> ? must be INTEGER
SELECT * FROM users LIMIT ? OFFSET ?;

-- 7.2. IN Operator -> ? must match column type
SELECT * FROM users WHERE username IN (?, ?, ?); -- ? are TEXT

-- 7.3. LIKE -> ? must be TEXT
SELECT * FROM users WHERE username LIKE ?;

-- 7.4. BETWEEN -> ? must match column type
SELECT * FROM users WHERE score BETWEEN ? AND ?; -- ? are INTEGER

-- 8.1. Cast to REAL -> Returns REAL
SELECT CAST(id AS REAL) FROM users;

-- 8.2. Cast to TEXT -> Returns TEXT
SELECT CAST(score AS TEXT) FROM users;

-- Assume table 'orders' exists with 'user_id' (INT NOT NULL)

-- 9.1. Left Join -> u.id is NOT NULL, o.user_id becomes Nullable INT
SELECT u.id, o.user_id 
FROM users u
LEFT JOIN orders o ON u.id = o.user_id;

-- 10.1. Mismatched Math
SELECT username + 5 FROM users; -- Error: TEXT + INT

-- 10.2. Mismatched Comparison
SELECT * FROM users WHERE score > 'high'; -- Error: INT > TEXT

-- 10.3. Mismatched CASE branches
SELECT CASE WHEN id > 5 THEN 1 ELSE 'a' END FROM users; -- Error: INT vs TEXT

-- 10.4. Invalid Parameter usage
SELECT * FROM users LIMIT 'five'; -- Error: LIMIT expects INT

-- 10.5. Boolean predicate check
SELECT * FROM users WHERE username; -- Error: username is TEXT, WHERE expects BOOL


-- 1.1. INSERT with VALUES
-- Inference: ? must match the type of col_a, col_b in schema order
INSERT INTO users (username, score) VALUES (?, ?);

-- 1.2. INSERT from SELECT
-- Validation: Type of 'old_users.name' must match 'users.username'
INSERT INTO users (username, score)
SELECT name, points FROM old_users;

-- 1.3. UPDATE with Bindings
-- Inference: ? must match type of 'score' column (INTEGER)
UPDATE users SET score = ? WHERE id = ?;

-- 1.4. UPDATE with Math
-- Validation: 'score' (INT) + ? must result in INT. Therefore ? is INT.
UPDATE users SET score = score + ?;

-- 1.5. INSERT ... RETURNING (Crucial for your project)
-- Logic: Acts like a SELECT. You must return the types of id (INT) and score (INT).
INSERT INTO users (username, score) VALUES (?, ?) RETURNING id, score;

-- 1.6. UPSERT (ON CONFLICT)
-- Validation: 'excluded.score' type must match 'score' column
INSERT INTO users (id, score) VALUES (?, ?)
ON CONFLICT(id) DO UPDATE SET score = excluded.score + 1;



-- 2.1. Simple CTE
-- Logic: 
-- 1. Infer 'recent_users' has columns: { name: TEXT, points: INT }
-- 2. Validate outer SELECT against those inferred types.
WITH recent_users AS (
    SELECT username AS name, score AS points FROM users WHERE id > 100
)
SELECT * FROM recent_users WHERE points > ?;

-- 2.2. Recursive CTE (Hard Mode)
-- Logic: The 'UNION ALL' requires the bottom SELECT to match types of top SELECT.
WITH RECURSIVE cnt(x) AS (
    SELECT 1                 -- Anchor: x is INT
    UNION ALL
    SELECT x + 1 FROM cnt    -- Recursive: INT + INT = INT (Matches)
    LIMIT 10
)
SELECT x FROM cnt;


-- 3.1. ROW_NUMBER / RANK
-- Return Type: ALWAYS INTEGER
SELECT username, ROW_NUMBER() OVER (ORDER BY score DESC) as rk FROM users;

-- 3.2. Aggregates as Window Functions
-- Logic: Same as normal aggregates. SUM(int) -> INT, AVG(int) -> REAL
SELECT 
    username, 
    AVG(score) OVER (PARTITION BY is_active) as avg_score 
FROM users;

-- 4.1. UNION
-- Validation: 
-- Column 1: users.id (INT) vs arch_users.id (INT) -> OK
-- Column 2: users.username (TEXT) vs arch_users.name (TEXT) -> OK
SELECT id, username FROM users
UNION
SELECT id, name FROM archived_users;


-- 5.1. datetime / date / time
-- Return Type: TEXT
SELECT datetime('now');

-- 5.2. strftime
-- Return Type: TEXT (even if you ask for %S seconds)
SELECT strftime('%Y-%m-%d', 'now');

-- 5.3. unixepoch
-- Return Type: INTEGER
SELECT unixepoch('now');

-- 6.1. json_extract
-- Return Type: ANY (In a strict system, usually treated as TEXT or a specific JSON type)
SELECT json_extract(metadata, '$.user_id') FROM logs;

-- 6.2. json_each / json_tree (Table Valued Functions)
-- Logic: These create a virtual table with columns (key, value, type, atom, id, parent, fullkey, path)
SELECT key, value FROM json_each(?) -- ? must be TEXT (json string)


-- 7.1. VALUES as a Table
-- Inference: Col1 is INT, Col2 is TEXT
SELECT column1, column2 FROM (VALUES (1, 'a'), (2, 'b'));