# Building schema

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

These are pretty similar. GraphQL and Postgres use different data type names, e.g varchar vs String. GraphQL uses upper camel case for type names, and camel case for all variable names, whilst Postgres conventionally uses snake case for everything. GraphQL signals nullability with '!' next to the data type, whilst Postgres uses 'not null' next to the type. Finally, the GraphQL Product type refers to a seller by value, and not by seller_id.

In order to pull this data from the Postgres database, I used the following query:
select table_name, column_name, data_type,
case is_nullable
when 'NO' then '!'
when 'YES' then ''
end as nullable
from information_schema.columns where table_schema = 'public'
group by table_name, column_name, data_type, nullable;

The information_schema table is a meta information table contained within a Postgres database. This database stores information about the tables, functions, triggers, types, etc. stores within the database itself. By using this table, I was able to fetch all columns, their types, nullabity, and the table they belonged to. The columns I select from the database are grouped primarily by the table they belong to. This is the sudo code I used to create the schema.

rows = "select table_name, column_name...".execute()

last_table_name = "";
current_graphql_type = "";
all_types = "";
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

# select all

Initially no filter select all exercises

The only data that this query required to work was the name of the table we select data from, and the fields that we want to select. I created a HashMap which translates

This was the easiest at all it required was formatting the table name and the fields we wanted to an SQL query. I did this by recursively traversing the fields. If the current field had children, the function was recursively called on all children, and the results were formatted into the parent query. If the current field had no children, they would simply return themselves as a select statement. I had to make sure that I converted all database fields from snake case back to camel case for GraphQL and vice versa.

    This was also very simple as it only had a depth of one. This select only selected from one table and asked for the fields of this table. This query also did not need to know anything about the database or the GraphQL schema.

# Adding a query which selects a single item based on ID.

The JSON build which is applied for a single element is different than that for multiple elements. In addition, there is an additional where clause at the end of the query so we select the correct row. In order to make this test pass, I added a HashMap field to my parser class. This HashMap goes from a String to a boolean (is_many). By doing this I was able to call the correct JSON build SQL code, and to know when to add the filter. I was able to access the ID that was passed to the GraphQL query through the AST.

# Adding a Join query

This increased complexity the most. So far the queries have worked even though they have been relatively blind to the database and the GraphQL schema. In order for joins to work, Poggers has to know when a field belongs to a table, and when a field is foreign. I initially used the graqphql-parser libraries schema parser. I fed the schema parser the containing the GraphQL schema string to see if I could make use of the existing library. After analyzing the data structure I realized that this would not be helpful for my implementation for a few reasons

    1. The fields were all stored as vectors with O(n) read access. This would mean that if I wanted to read e.g the employee fields, I would need to iterate over all types before I found employee. Something like a HashMap would be far more suitable.
    2. The data is not circular/recursive in any fashion. A query might select some fields from one type, jump to another type and select some fields, and get some nested fields from a third type. Something involving pointers/a graph would be very helpful for this. This would allow you to process the query, and make calls to foreign tables recursively, passing that query information through the pointer to that data.
    3. I needed to embed my own context in to the queries, and didn't need a lot of the provided context. Each type should also include the corresponding table it maps to, whether a foreign query returns one or many, etc. There was also a lot of unnecessary context that the parser included, such as row and column position of each field, which I did not need.

I initially tried to create a Graph like structure, where each node would be a type containing a set of terminal fields, and a set of pointers to other types. This proved to be very challenging due to Rust's ownership model. I also tried to create a vector based Graph, where each item was a type which contained its own set of vector indices and it's relation to these types, but rust was also very unhappy with this due to the borrow checker. I caved in and downloaded a package called petcrate which allows you to create a directional Graph. Each node in this directional Graph stores a set containing all names of all terminal fields, and the table the type corresponds to. A node has an edge to another node if one has a foreign key to the other. Each edge stores the name of the GraphQL field it corresponds to. As an example, if employees have a foreign key to departments, departments have an edge where (graphql_field_name) is "employees". When we encounter a field, we first check the current nodes terminal fields hashset for presence. If it's there, this GraphQL field is processed as a column of the table. Otherwise we iterate over all edges of the node until we find a node who's graphql_field_name matches this field. The edge also stores other information needed to construct the query: the directionality of the relation (one to many or many to one) and the column they are joined on. We also use the information stored on the node endpoints of this edge (e.g table_name) in order to join the correct tables together. Who would've thought that a graph based representation of a graph based query language would work well. One of the performance assumptions of this implementation is that tables generally have more of their own columns rather than foreign keys. Identifying your own columns happens with O(1) complexity through the hashset, and is always done before traversing edges finding an edge with the matching field name. This implementation might be slow with tables with lots of foreign keys, as each join has complexity O(f) complexity, where f is the number of foreign keys a table has. This also assumes that the number of foreign keys is relatively small, so that the runtime complexity difference isn't too significant, and hashset overheads such as the hashing function are still relevant.

Migrating from graphql_parser to async_graphql_parser
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

In async_graphql_parser however, every document element is wrapped in generic Positioned struct:

pub struct Positioned<T: ?Sized> {
pub pos: Pos,
pub node: T,
}
As an example, this struct represents the root of the GraphQL query

pub struct SelectionSet {
pub items: Vec<Positioned<Selection>>,
}

What this means is that when iterating over the queries I need to call e.g item.node.name and not just item.name. This isn't the end of the world, but can make for some ugly one liners, e.g:

for selection in &field.node.selection_set.node.items {

which before would've just been
for selection in &field.selection_set.items {

But as this is a performance critical project I should use async_graphql_parser even for the small performance decrease.

Migrating was a lot easier than I thought, even though it required replacing all function signatures and anything related to accessing query data. This was partly due to the SQL logic not changing at all, the internal representation of the schema not changing, and the fact that both parsers represent the same specification albeit differently. Rust's compiler is also very strict, so a lot of the replacement was simply going to errors with my editor, fixing it, and then going to the next error. This made me appreciate the strongly typed nature of rust and the strict memory rules, as migrating this library would have been much more hands on in a dynamically typed language with no library specific errors, and just syntax ones.

Three way join
My implementation didn't yet generalize to a three way join. I realized that the difference with a nested join and a singular join is that singular joins call to_json whilst nested joins don't call to_json. Another difference is how nested joins define their key string. JSON is structured as a series of {key: values}, and the nested join defines it's key before its value, whilst a non nested join defines its key name after the value. Everything in the center is the same, only the surroundings and the ending are different.

## nested join

'@foreignTables'::text,
(
...
)

## non nested join

to_json (
...
) as "@foreignTables"

In order to surround the query with the correct text, the query constructor needs to know if it's in a nested or non nested join. I decided to add a boolean parameter is_nested_join to the build_foreign_field function. Then this method surrounds the query conditionally based on the value of is_nested_join. NextI had to figure out when the boolean should be true and false.

build_selection is called on the root of the query. build_foreign_field would make no sense to call, as the root query is the first table we select from, so it doesn't have any children. Therefore any joins performed by build_selection (calls to build_foreign_field) would not be a nested join, so is_nested_join should be false.

build_foreign_field is recursively called by itself to perform any nested joins. Logically, if we're in a build_foreign_field call, this means that any subsequent joins would be nested, so any recursive call will have is_nested_join be set as true

Luckily, a three way join generalized to a six way join. Although I haven't tested a larger join, I have reason to believe that my implementation works for any joins with more than six tables, as the rules for any joins larger than two do not seem to change.

#

# refactoring

All the logic related to query handling was being done in a single class. This became very uncomfortable to work with, as it was difficult to follow logic flows, remember what happens where etc. even though I wrote the code myself. I had considered separating different visitors in to different files/classes, but this didn't wouldn't actually address the original problem God Object problem. I needed more abstraction in the visitor in my implementation, so it would be easier to reason about the overall behaviour of the system, rather than focus on individual implmentations. I finally came up with a good idea: I should abstract away all the SQL building from the visitor. The visitor should still call functions related to building queries, but the visitor shouldn't need to know anything about the underlying query language or database protocol. I created a trait (equivalent to an interface in object oriented languages) named GraphQLQueryBuilder, and a struct which implemented this trait. I then added a struct field to the visitor, which had to be a GraphQLQueryBuilder (for now this has to be the Postgres one as it's the only one that implements it, but others may do so in the future). I then cut and paste SQL snippets from the visitor, added an apt name in the trait, and moved the cut code to this new function. The visitor would then call the GraphQLQueryBuilder's to append the SQL to this query instead of doing so itself. I would then run all tests, and if they passes git commit. After doing this for all snippets, I was able to extract all SQL away from the visitor. This lead to a very nice outcome.

Prior to the refactoring, the visitor was about 270 lines long. This was shrunk down to only about 170 lines. The new SQL builder file was 177 lines. The total number of lines of code increased due to the functions definitions and method calls which weren't there before.

# The benefits of this refactoring:

    * Much easier to reason about how the visitor works and how the visitor uses information about the query, as there are no SQL implementation details
    * The calls to the SQL builders are self documenting: for instance, it is much easier to understand that this line of code builds the query which selects a leaf (terminal nodes) data.
    		SQL::build_terminal_field(&mut s, field_name);

    		rather than this:

    		s.push_str("to_json((__local_0__.\"");
    		s.push_str(&field_name.to_case(Case::Snake));
    		s.push_str("\")) as \"");
    		s.push_str(field_name);
    		s.push_str("\",\n");
    * It was much easier to notice repeated SQL logic, to allow for function composition, less repetition and more reuse.

    * This will make it much easier to support other database dialects. I will not need to implement another visitor, I will only need to implement the query building defined by the trait. I will probably need to change some of the function calls, as the trait (interface) function calls are defined based on Postgres, and not any other query language or database dialect. Adding support for another SQL dialect such as MYSQL or Oracle might be very feasible, and even some Graph databases such as MongoDB.

# Many to one

After doing this refactoring I decided to implement the many to one join. E.g instead of selecting employees for department, we select the department for every employee. Although SQL might not differentiate between one to many and many to one, due to the nested structure of GraphQL there is a difference.

One to many
query {
departments {
id
employee {
name
}
}
}

Example response for above

{
"departments" [
"id": 1,
"employees" [
{"name": "Bob"},
{"name": "Rob"},
]
]
}

query {
employees {
name
department {
id
}
}
}

Example response for above

"employees": {
name: "Bob",
"department": {
id: 1
},
name: "Rob",
"department": {
id: 1
},
}

This was pretty easy to implement. Syntactically this was very similar to the one to many join, but without additional surroundings that one to many has

many to one:
select json_build_object(
...  
 workout_plan_id\" = **local_1**.\"id\")

one to many

    select coalesce(
    	(

    		select json_agg(__local_1__.\"object\")
    			from (
    				select json_build_object(
    					...
    				workout_plan_id\" = __local_1__.\"id\")
    			),
    		'[]'::json
    	)
    )

There are separate methods for closing and opening a join query. It can be thought of as a Hamburger, you can't just put both together before you put food on the bottom bun. The join_query_header is like the bottom bun which is called when we first encounter a foreign join. We then add all the fillings (any table fields this table had) and the query to fetch any foreign fields values recursively. Only once those have been added, can we add the top bun (join_query_closer). I added an extra parameter to both header and closer, one_to_many. This tells the method whether it should add the extra wrapping or not. The one_to_many value is gotten from the edge weight connecting the two types in our schema graph.

