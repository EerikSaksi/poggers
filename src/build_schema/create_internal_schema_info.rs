enum GraphQLSelection<'a> {
    Field(String), 
    Composite(String, Vec<&'a GraphQLSelection<'a>>)
}
