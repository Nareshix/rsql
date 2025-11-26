1. nearly all select statemnets start with Query. It is only in Body where u need to parse the Select() enum Which subsequently contains Select struct

2. In Select struct, these are the only things u need to focus.
   - projection
   - from
   - selection (basically ur WHERE.)
   - group_by
   - having
   - distinct
   - (note order by and limit is in above Query)