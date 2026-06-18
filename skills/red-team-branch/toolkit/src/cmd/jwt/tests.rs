#[cfg(test)]
mod tests {
    use crate::cmd::jwt::{attack, parse, report};

    const VALID_JWT: &str = "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkwIn0.Zm9vYmFy";

    #[test]
    fn parse_valid_jwt() {
        let jwt = parse::parse(VALID_JWT).unwrap();
        assert_eq!(jwt.header_json["alg"], "RS256");
        assert_eq!(jwt.header_json["typ"], "JWT");
    }

    #[test]
    fn none_attack_removes_signature() {
        let jwt = parse::parse(VALID_JWT).unwrap();
        let attacks = attack::all_attacks(&jwt, "HS256", None, None);
        let none = attacks.iter().find(|a| a.name == "alg-none").unwrap();
        assert!(none.token.ends_with('.'));
    }

    #[test]
    fn blank_secret_generates_signature() {
        let jwt = parse::parse(VALID_JWT).unwrap();
        let attacks = attack::all_attacks(&jwt, "HS256", None, None);
        let blank = attacks.iter().find(|a| a.name == "blank-secret").unwrap();
        let parts: Vec<&str> = blank.token.splitn(3, '.').collect();
        assert_eq!(parts.len(), 3);
        assert!(!parts[2].is_empty());
    }

    #[test]
    fn confusion_attack_uses_public_key() {
        let jwt = parse::parse(VALID_JWT).unwrap();
        let attacks = attack::all_attacks(&jwt, "HS256", Some("pubkey"), None);
        let confusion = attacks.iter().find(|a| a.name == "alg-confusion").unwrap();
        assert!(confusion.token.contains('.'));
    }

    #[test]
    fn kid_injection_forces_symmetric_alg() {
        let jwt = parse::parse(VALID_JWT).unwrap();
        let attacks = attack::all_attacks(&jwt, "HS256", None, Some("/dev/null"));
        let kid = attacks.iter().find(|a| a.name == "kid-injection").unwrap();
        assert!(kid.header.contains(r#""alg":"HS256""#), "kid attack must claim HS256");
        assert!(kid.header.contains("/dev/null"));
    }

    #[test]
    fn exploit_requires_control_rejection() {
        let jwt = parse::parse(VALID_JWT).unwrap();
        let atk = &attack::all_attacks(&jwt, "HS256", None, None)[0];
        // Endpoint rejects invalid sig (401) but accepts the forgery (200): real bypass.
        assert_eq!(report::classify(atk, 200, 401).verdict, "VULNERABLE");
        // Endpoint accepts the invalid-sig control too: it ignores signatures, not provable.
        assert_eq!(report::classify(atk, 200, 200).verdict, "BLOCKED");
    }
}
