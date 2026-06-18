mod common;
mod gql_fixtures;

// Pointing gql at a non-GraphQL 200 endpoint must NOT be flagged: the response
// carries no `data`. Under the old status<400 oracle this was a false positive.
#[test]
fn gql_does_not_false_positive_on_non_graphql_endpoint() {
    let port = common::start_server();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &[
        "gql", "--url", "/", "--endpoint", "/echo",
        "--introspection", "--batching", "--aliasing",
    ]);
    assert_eq!(v["probe"], "gql");
    assert_eq!(v["verdict"], "GRAPHQL SAFE");
    assert_eq!(v["introspection_enabled"], false);
    assert_eq!(v["batching_enabled"], false);
    assert_eq!(v["get_query_enabled"], false);
}

// Negative control: an endpoint that returns a 2-element ARRAY where every
// element is a pure error (no executed `data`) is a rejected batch, not an
// enabled one. A bare array-length>=2 oracle false-positives here.
#[test]
fn gql_batch_array_of_errors_is_not_batching() {
    let port = gql_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &[
        "gql", "--url", "/", "--endpoint", "/batch-errors", "--batching",
    ]);
    assert_eq!(
        v["batching_enabled"], false,
        "array of errors carries no executed data; must not be flagged: {v}"
    );
}

// Detection: a real GraphQL server (introspection schema + per-element batch
// data) must be flagged on both introspection and batching.
#[test]
fn gql_flags_vulnerable_graphql_server() {
    let port = gql_fixtures::start_fixture();
    let cfg = common::write_config(port);
    let v = common::run_rtk(&cfg, &[
        "gql", "--url", "/", "--endpoint", "/vuln", "--introspection", "--batching",
    ]);
    assert_eq!(v["introspection_enabled"], true, "schema types returned: {v}");
    assert_eq!(v["batching_enabled"], true, "each element carries data: {v}");
    assert_eq!(v["verdict"], "GRAPHQL MISCONFIGURATION");
}
