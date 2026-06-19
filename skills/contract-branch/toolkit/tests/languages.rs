mod common;
use common::{candidate, run_ctk, Repo};

/// Go: exported (capitalized) symbols are the contract. Removing an exported
/// func and an exported struct field are breaking; an unexported change is not.
#[test]
fn go_exported_contract() {
    let repo = Repo::new();
    repo.write(
        "svc.go",
        "package x\ntype Config struct { Host string; secret int }\nfunc Serve(a int) int { return a }\n",
    );
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.write(
        "svc.go",
        "package x\ntype Config struct { secret int }\nfunc serve(a int) int { return a }\n",
    );
    repo.commit_all("drop Host, unexport Serve");

    let v = run_ctk(repo.path(), &["--base", "main", "assess"]);
    assert_eq!(v["semver_impact"], "major");
    assert_eq!(candidate(&v, "Config::Host").unwrap()["reason"], "REMOVED");
    assert!(candidate(&v, "Serve").is_some(), "unexporting Serve removes it from the contract");
}

/// TypeScript: a public method of an exported class is part of the contract;
/// removing it breaks callers, while a `private` method is invisible.
#[test]
fn ts_class_method_contract() {
    let repo = Repo::new();
    repo.write(
        "api.ts",
        "export class Service {\n  handle(a: number): number { return a }\n  private secret(): void {}\n}\n",
    );
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.write(
        "api.ts",
        "export class Service {\n  private secret(): void {}\n}\n",
    );
    repo.commit_all("remove public handle");

    let v = run_ctk(repo.path(), &["--base", "main", "assess"]);
    assert_eq!(v["semver_impact"], "major");
    assert_eq!(candidate(&v, "Service::handle").unwrap()["reason"], "REMOVED");
    // the private method never enters the contract, so changing it is silent.
    assert!(candidate(&v, "secret").is_none());
}

/// Python: a public function gaining a required parameter is a signature change;
/// an underscore-prefixed function is private and ignored.
#[test]
fn python_contract() {
    let repo = Repo::new();
    repo.write("svc.py", "def handler(a):\n    return a\ndef _helper():\n    pass\n");
    repo.commit_all("base");

    repo.checkout_new("feature");
    repo.write("svc.py", "def handler(a, b):\n    return a\n", );
    repo.commit_all("add required param, drop private helper");

    let v = run_ctk(repo.path(), &["--base", "main", "assess"]);
    assert_eq!(candidate(&v, "handler").unwrap()["reason"], "SIGNATURE_CHANGED");
    assert!(candidate(&v, "_helper").is_none(), "private function is not contract");
}
