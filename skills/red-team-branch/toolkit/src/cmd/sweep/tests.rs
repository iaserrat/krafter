use super::classify::classify;
use crate::http::Outcome;

fn oc(status: u16, sha: &str) -> Outcome {
    Outcome {
        status,
        body_sha8: sha.into(),
        ..Default::default()
    }
}

#[test]
fn anon_reading_same_body_is_public_not_idor() {
    assert_eq!(
        classify("1".into(), oc(200, "a"), oc(200, "a"), None).class,
        "public"
    );
}

#[test]
fn both_actors_read_same_private_record_is_proven() {
    assert_eq!(
        classify("1".into(), oc(200, "a"), oc(401, "z"), Some(oc(200, "a"))).class,
        "cross_user_proven"
    );
}

#[test]
fn access_controlled_without_compare_is_accessible() {
    assert_eq!(
        classify("1".into(), oc(200, "a"), oc(401, "z"), None).class,
        "accessible"
    );
}

#[test]
fn compare_denied_is_scoped_not_accessible() {
    assert_eq!(
        classify("1".into(), oc(200, "a"), oc(401, "z"), Some(oc(403, "x"))).class,
        "scoped"
    );
}

#[test]
fn no_access_is_denied() {
    assert_eq!(
        classify("1".into(), oc(403, "x"), oc(401, "z"), None).class,
        "denied"
    );
}

#[test]
fn sensitive_field_names() {
    let hit = classify(
        "1".into(),
        Outcome {
            status: 200,
            body_raw: br#"{"ssn":"x","name":"n"}"#.to_vec(),
            ..Default::default()
        },
        oc(401, "z"),
        None,
    );
    assert!(hit.sensitive.iter().any(|k| k == "ssn"));
}
