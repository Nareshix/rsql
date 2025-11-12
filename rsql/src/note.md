some error handling can be omitted depending on how u structure ur code
for instance error will be return if u are using a null pointer for a funciton. but the program does not proceed in the first place if null pointer was present (e.g. db open, closes when ppdb is NULL)

CString::new states that the rust code given must not contain a null pointer inside it, '\0' (hence the unsafe)
handle those cases as well. currently i unwarap it. 





sqlite3_busy_timeout vs thread sleep
replace all expect and unwrap with  proper error hadnling

for some of the opeartions involving strings i added -1 which cause sqlite (internally) to perform O(n) operation to extract the string which could be avoided if i give it the length beforehand (O(1))


have a execute_all feature with mulitp;le sql statement