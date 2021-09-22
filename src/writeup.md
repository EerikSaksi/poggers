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

