GraphQL schemas can be considered like Menus for data. They let the client know what queries exist, what types these queries return, what fields types have, and how different types are related. I saw this as the first step to my project, as clients would need to know what they could fetch before I could properly test them. Instead of expecting a user of my software to manually write a schema which is compatible with their database, and to update it whenever their database updates, I will automatically generate it for a given database. 

To better illustrate what my goal is, here is a hypothetical database implementation:

create table seller (
	id integer primary key,
	name varchar 
);
create table product (
	id integer primary key, 
	listed_price float not null,
	seller_id integer references seller(id)
);

Sellers have a name and an id, and products have a name, a price and a foreign key referencing a seller. This would be these two tables as GraphQL types:

type Seller {
	id: Int!
	name: String
}
type Product {
	id: Int!
	listedPrice: Float!
	seller: Seller
}

These are pretty similar. GraphQL and Postgres use different data type names, e.g varchar vs String. GraphQL uses upper camel case for type names, and camel case for all variable names, whilst Postgres conventionally uses snake case for everything. GraphQL signals nullability with '!' next to the data type, whilst Postgres uses 'not null' next to the type. Finally, the GraphQL Product type refers to a seller by value, and not by seller\_id.

In order to pull this data from the Postgres database, I used the following query:
select table\_name, column\_name, data\_type,
		case is\_nullable
				when 'NO' then '!'
				when 'YES' then ''
		end as nullable
			from information\_schema.columns where table\_schema = 'public' 
				group by table\_name, column\_name, data\_type, nullable;

The information\_schema table is a meta information table contained within a Postgres database. This database stores information about the tables, functions, triggers, types, etc. stores within the database itself. By using this table, I was able to fetch all columns, their types, nullabity, and the table they belonged to. The columns I select from the database are grouped primarily by the table they belong to. This is the sudo code I used to create the schema.


rows = "select table\_name, column\_name...".execute()


last\_table\_name = "";
current\_graphql\_type = "";
all\_types = "";
for row in rows {
		//current_graphql_type will receive no more fields (as we group columns by table_name there will be no more fields for this table)
		if row.table_name != last_table_name {
				//close the type with closing brace
				current_graphql_type += "\n}"

				//add final current type to all types
				all_types += current_graphql_type


				last_table_name = row.table_name

				//current_graphql_type should be reinitialized with no fields with the current
				current_graphql_type += "type {last_table_name}{\n"
		}
}

With regards to queries, initially I wanted to implement my own parser. I saw this as beneficial, as it would allow me to resolve values in one pass. After realizing how hard this would be, and how I would need to keep updating this parser as the GraphQL spec changed, I decided that it would make more sense to simply to download a parser. I benchmarked the parser and was amazed at it's performance. It was able to parse queries in a matter of microseconds. Thus I decided that I should just parse requests, and then visit the generated AST.

Next I decided to try to implement a generalized query parser. This simply visited matched enum types, and then visited the various types. I only implemented visitors for query types, and not mutations and subscriptions. I implemented some tests based on the explain that Postgraphile generated for GraphQL queries. I was able to create a query for 


I implemented a visitor design pattern which visited different selections of fields

I used a test driven development in order to progressively add features

Initially no filter select all exercises
	This was the easiest at all it required was formatting the table name and the fields we wanted to an SQL query. I did this by recursively traversing the fields. If the current field had children, the function was recursively called on all children, and the results were formatted into the parent query. If the current field had no children, they would simply return themselves as a select statement. I had to make sure that I converted all database fields from snake case back to camel case for GraphQL and vice versa.

	This was also very simple as it only had a depth of one. This select only selected from one table and asked for the fields of this table. This query also did not need to know anything about the database or the GraphQL schema. 

Adding a query which selects a single item based on ID.
	The JSON build which is applied for a single element is different than that for multiple elements. In addition, there is an additional where clause at the end of the query so we select the correct row. In order to make this test pass, I added a HashMap field to my parser class. This HashMap goes from a String to a boolean (is_many). By doing this I was able to call the correct JSON build SQL code, and to know when to add the filter. I was able to access the ID that was passed to the GraphQL query through the AST. 

Adding a Join query

This increased complexity the most. So far the queries have worked even though they have been relatively blind to the database and the GraphQL schema. In order for joins to work, Poggers has to know when a field belongs to a table, and when a field is foreign. I initially used the graqphql-parser libraries schema parser. I fed the schema parser the containing the GraphQL schema string to see if I could make use of the existing library. After analyzing the data structure I realized that this would not be helpful for my implementation for a few reasons

	1. The fields were all stored as vectors with O(n) read access. This would mean that if I wanted to read e.g the employee fields, I would need to iterate over all types before I found employee. Something like a HashMap would be far more suitable. 
	2. The data is not circular/recursive in any fashion. A query might select some fields from one type, jump to another type and select some fields, and get some nested fields from a third type. Something involving pointers/a graph would be very helpful for this. This would allow you to process the query, and make calls to foreign tables recursively, passing that query information through the pointer to that data.
	3. I needed to embed my own context in to the queries, and didn't need a lot of the provided context. Each type should also include the corresponding table it maps to, whether a foreign query returns one or many, etc. There was also a lot of unnecessary context that the parser included, such as row and column position of each field, which I did not need.

I initially tried to create a Graph like structure, where each node would be a type containing a set of terminal fields, and a set of pointers to other types. This proved to be very challenging due to Rust's ownership model. I also tried to create a vector based Graph, where each item was a type which contained its own set of vector indices and it's relation to these types, but rust was also very unhappy with this due to the borrow checker. I caved in and downloaded a package called petcrate which allows you to create a directional Graph. Each node in this directional Graph stores a set containing all names of all terminal fields, and the table the type corresponds to. A node has an edge to another node if one has a foreign key to the other. This edge also contains information about this relation (one to many, many to one, etc) as well as the name of this field (if one to many the field is capitalized otherwise not). 
