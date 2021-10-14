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

# Adding a Join query

This increased complexity the most. So far the queries have worked even though they have been relatively blind to the database and the GraphQL schema. In order for joins to work, Poggers has to know when a field belongs to a table, and when a field is foreign. I initially used the graqphql-parser libraries schema parser. I fed the schema parser the containing the GraphQL schema string to see if I could make use of the existing library. After analyzing the data structure I realized that this would not be helpful for my implementation for a few reasons

	1. The fields were all stored as vectors with O(n) read access. This would mean that if I wanted to read e.g the employee fields, I would need to iterate over all types before I found employee. Something like a HashMap would be far more suitable. 
	2. The data is not circular/recursive in any fashion. A query might select some fields from one type, jump to another type and select some fields, and get some nested fields from a third type. Something involving pointers/a graph would be very helpful for this. This would allow you to process the query, and make calls to foreign tables recursively, passing that query information through the pointer to that data.
	3. I needed to embed my own context in to the queries, and didn't need a lot of the provided context. Each type should also include the corresponding table it maps to, whether a foreign query returns one or many, etc. There was also a lot of unnecessary context that the parser included, such as row and column position of each field, which I did not need.

I initially tried to create a Graph like structure, where each node would be a type containing a set of terminal fields, and a set of pointers to other types. This proved to be very challenging due to Rust's ownership model. I also tried to create a vector based Graph, where each item was a type which contained its own set of vector indices and it's relation to these types, but rust was also very unhappy with this due to the borrow checker. I caved in and downloaded a package called petcrate which allows you to create a directional Graph. Each node in this directional Graph stores a set containing all names of all terminal fields, and the table the type corresponds to. A node has an edge to another node if one has a foreign key to the other. Each edge stores the name of the GraphQL field it corresponds to. As an example, if employees have a foreign key to departments, departments have an edge where (graphql\_field\_name) is "employees". When we encounter a field, we first check the current nodes terminal fields hashset for presence. If it's there, this GraphQL field is processed as a column of the table. Otherwise we iterate over all edges of the node until we find a node who's graphql\_field\_name matches this field. The edge also stores other information needed to construct the query: the directionality of the relation (one to many or many to one) and the column they are joined on. We also use the information stored on the node endpoints of this edge (e.g table\_name) in order to join the correct tables together. Who would've thought that a graph based representation of a graph based query language would work well. One of the performance assumptions of this implementation is that tables generally have more of their own columns rather than foreign keys. Identifying your own columns happens with O(1) complexity through the hashset, and is always done before traversing edges finding an edge with the matching field name. This implementation might be slow with tables with lots of foreign keys, as each join has complexity O(f) complexity, where f is the number of foreign keys a table has. This also assumes that the number of foreign keys is relatively small, so that the runtime complexity difference isn't too significant, and hashset overheads such as the hashing function are still relevant.


Migrating from graphql\_parser to async\_graphql\_parser
https://colab.research.google.com/drive/1JcsPot2k_03IYVcFG-ifNNiW0EeHjXqo

One downside of this implementation is slight loss in readability. Both the Juniper parser and the async GraphQL parser store the positions of the fields (row and column). This data is irrelevant to me. The Juniper parser defines it like so:

pub struct Field<'a, T: Text<'a>> {
    pub position: Pos,
    pub alias: Option<T::Value>,
    pub name: T::Value,
    pub arguments: Vec<(T::Value, Value<'a, T>)>,
    pub directives: Vec<Directive<'a, T>>,
    pub selection_set: SelectionSet<'a, T>,
}

In this case, the position is simply a field that I don't currently need, much like alias, name and directives.

In async\_graphql\_parser however, every document element is wrapped in generic Positioned struct:

pub struct Positioned<T: ?Sized> {
    pub pos: Pos,
    pub node: T,
}
As an example, this struct represents the root of the GraphQL query 

pub struct SelectionSet {
    pub items: Vec<Positioned<Selection>>,
}

What this means is that when iterating over the queries I need to call e.g item.node.name and not just item.name. This isn't the end of the world, but can make for some ugly one liners, e.g:

for selection in &field.node.selection\_set.node.items {

which before would've just been
for selection in &field.selection\_set.items {

But as this is a performance critical project I should use async\_graphql\_parser even for the small performance decrease.

Migrating was a lot easier than I thought, even though it required replacing all function signatures and anything related to accessing query data. This was partly due to the SQL logic not changing at all, the internal representation of the schema not changing, and the fact that both parsers represent the same specification albeit differently. Rust's compiler is also very strict, so a lot of the replacement was simply going to errors with my editor, fixing it, and then going to the next error. This made me appreciate the strongly typed nature of rust and the strict memory rules, as migrating this library would have been much more hands on in a dynamically typed language with no library specific errors, and just syntax ones. 



Three way join

My implementation didn't yet generalize to there


