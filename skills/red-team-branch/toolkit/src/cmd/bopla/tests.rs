use super::field;
use serde_json::Value;

#[test]
fn parses_typed_then_falls_back_to_string() {
    assert_eq!(
        field::parse("is_admin=true").unwrap(),
        ("is_admin".into(), Value::Bool(true))
    );
    assert_eq!(field::parse("n=5").unwrap().1, serde_json::json!(5));
    assert_eq!(
        field::parse("role=admin").unwrap(),
        ("role".into(), Value::String("admin".into()))
    );
    assert!(field::parse("noeq").is_err());
}

#[test]
fn persistence_requires_exact_echo() {
    assert!(field::persisted(
        br#"{"is_admin":true}"#,
        "is_admin",
        &Value::Bool(true)
    ));
    assert!(!field::persisted(
        br#"{"is_admin":false}"#,
        "is_admin",
        &Value::Bool(true)
    ));
    assert!(!field::persisted(
        br#"{"other":1}"#,
        "is_admin",
        &Value::Bool(true)
    ));
    assert!(!field::persisted(
        b"<html>not json</html>",
        "is_admin",
        &Value::Bool(true)
    ));
}
