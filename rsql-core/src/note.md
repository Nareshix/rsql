some error handling can be omitted depending on how u structure ur code
for instance error will be return if u are using a null pointer for a funciton. but the program does not proceed in the first place if null pointer was present (e.g. db open, closes when ppdb is NULL)

CString::new states that the rust code given must not contain a null pointer inside it, '\0' (hence the unsafe)
handle those cases as well. currently i unwarap it. 


SQLITE_LIMIT_VARIABLE_NUMBER for bulk insert


when hit wih api,

1. checks cache
2. if dont exist, prepares staement and cache it
3. run write or read operaion (loop the cursor)
4. after ur done reset the staement. don finalise the statement


looping (bulk insert large data in batches. never loop select statements, they can awlays be done in sql query and u loop the cursor)
1. checks cache
2. if dont exist, prepares staement and cache it
3. run the loop  and in each loop reset the statement
4. dont finalise the statemnet




replace all expect and unwrap with  proper error hadnling

for some of the opeartions involving strings i added -1 which cause sqlite (internally) to perform O(n) operation to extract the string which could be avoided if i give it the length beforehand (O(1))


have a execute_all feature with mulitp;le sql statement

rn macros silently fail for unique constraint (this is where the compile time checks come in eventually)

also, rn texts (and blobs) are converted each time (c to rust). maybe can use c string directly via lifetimes?


query! macro might work as expected due to rust's lifetime. its quite a difficult task so ill leave it for now and focus it on the future (or maybe not)

offer an option that acutally allows teh desired syntax (let result = query!()) but do inform them that it is inefficent as it colelcts them into a vec ro other data types

tuple expresison and bind parameter compile time doesnt check

consider using a vector instead of hashmap. realistically an app usulaly wouldnt have that many cached statements in the first place
