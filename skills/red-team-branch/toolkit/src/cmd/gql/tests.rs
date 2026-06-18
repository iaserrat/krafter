use super::probe;

#[test]
fn introspection_query_is_valid_graphql() {
    assert!(probe::INTROSPECTION_QUERY.contains("__schema"));
    assert!(probe::INTROSPECTION_QUERY.starts_with("query"));
}
