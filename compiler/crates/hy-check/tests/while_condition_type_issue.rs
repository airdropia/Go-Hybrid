//! While loop condition type mismatch — condition must be boolean (HYB-E1002).
//!
//! Go requires `for` loop conditions to be boolean. Hybrid must too
//! (GUARANTEES Row 13).

use hy_check::{check_file, Verdict};
use hy_diag::Severity;
use hy_syntax::parse_file;

fn check(src: &str) -> hy_check::CheckResult {
    let path = "inline.hy";
    let ast = parse_file(path, src)
        .unwrap_or_else(|diags| panic!("must parse (parser contract): {:?}", diags));
    check_file(&ast)
}

fn codes(r: &hy_check::CheckResult) -> Vec<String> {
    r.diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.clone())
        .collect()
}

fn assert_code(src: &str, want: &str) {
    let r = check(src);
    let cs = codes(&r);
    assert_eq!(
        cs,
        vec![want.to_string()],
        "expected [{want}] for:\n{src}\ngot {cs:?}"
    );
}

fn assert_ok(src: &str) {
    let r = check(src);
    let cs = codes(&r);
    assert!(cs.is_empty(), "expected 0 diagnostics, got {cs:?} for:\n{src}");
}

// ---------------------------------------------------------------------------
// Reject: non-boolean while loop condition
// ---------------------------------------------------------------------------

#[test]
fn while_int_condition_is_rejected() {
    // `for x { }` where x is int — condition must be boolean.
    assert_code(
        "fn main() {\n    x := 1\n    for x { }\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn return_string_condition_is_rejected() {
    // `for "hello" { }` — string used as boolean condition.
    assert_code(
        "fn main() {\n    for \"hello\" { }\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Accept: boolean while loop condition
// ---------------------------------------------------------------------------

#[test]
fn while_bool_condition_is_accepted() {
    // `for true { }` — boolean condition.
    assert_ok(
        "fn main() {\n    for true { }\n}\n",
    );
}

#[test]
fn while_bool_var_condition_is_accepted() {
    // `for ok { }` where ok is bool — boolean condition.
    assert_ok(
        "fn main() {\n    ok := true\n    for ok { }\n}\n",
    );
}
