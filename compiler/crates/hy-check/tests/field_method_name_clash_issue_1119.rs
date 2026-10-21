//! #1119 — Reject a struct field and a method with the same name (HYB-E1002).
//!
//! Go rejects a struct type that has a field and a method with the same name
//! (`field and method with the same name n`). `hyb check` must reject it
//! fail-closed so the emitted Go always builds (GUARANTEES.md Row 13).

use hy_check::{check_file, CheckResult, Verdict};
use hy_diag::Severity;
use hy_syntax::parse_file;

fn check(src: &str) -> CheckResult {
    let path = "inline.hy";
    let ast = parse_file(path, src)
        .unwrap_or_else(|diags| panic!("must parse (parser contract): {:?}", diags));
    check_file(&ast)
}

fn codes(r: &CheckResult) -> Vec<String> {
    r.diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.clone())
        .collect()
}

fn assert_reject_e1002(src: &str, what: &str) {
    let r = check(src);
    assert_eq!(
        r.verdict,
        Verdict::Reject,
        "expected REJECT, got {:?} for:\n{}",
        r.verdict, src
    );
    assert!(
        codes(&r).iter().any(|c| c == "HYB-E1002"),
        "expected HYB-E1002, got {:?} for:\n{}",
        codes(&r),
        src
    );
}

fn assert_accept(src: &str) {
    let r = check(src);
    assert_eq!(
        r.verdict,
        Verdict::Accept,
        "expected ACCEPT, got {:?} for:\n{}",
        r.verdict, src
    );
}

// --- Reject: value receiver method same name as field ---

#[test]
fn field_and_value_method_same_name_rejects() {
    assert_reject_e1002(
        "struct counter {\n    n int\n}\n\nfn (c counter) n() int {\n    return 1\n}\n\nfn main() {\n    c := counter{n: 2}\n    fmt.println(\"{c.n}\")\n}\n",
        "field `n` + value method `n`",
    );
}

// --- Reject: pointer receiver method same name as field ---

#[test]
fn field_and_pointer_method_same_name_rejects() {
    assert_reject_e1002(
        "struct counter {\n    n int\n}\n\nfn (c *counter) n() int {\n    return 1\n}\n\nfn main() {\n    c := counter{n: 2}\n    fmt.println(\"{c.n}\")\n}\n",
        "field `n` + pointer method `n`",
    );
}

// --- Accept: distinct field and method names ---

#[test]
fn distinct_field_and_method_names_accept() {
    assert_accept(
        "struct counter {\n    n int\n}\n\nfn (c counter) get() int {\n    return c.n\n}\n\nfn main() {\n    c := counter{n: 2}\n    fmt.println(\"{c.get()}\")\n}\n",
    );
}

// --- Accept: method name different from field name ---

#[test]
fn method_name_different_from_field_accept() {
    assert_accept(
        "struct counter {\n    count int\n}\n\nfn (c counter) n() int {\n    return 1\n}\n\nfn main() {\n    c := counter{count: 2}\n    fmt.println(\"{c.n()}\")\n}\n",
    );
}
