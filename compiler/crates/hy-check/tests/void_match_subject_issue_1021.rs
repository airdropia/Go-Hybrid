//! #1021 — Void match subjects and conflicting pattern literal types bypass
//! front-end checks (HYB-E1002).
//!
//! Match statement subjects and arm pattern literals are not type-checked:
//! void calls and 0-value builtins pass as match subjects, and literal
//! patterns of incompatible types are accepted. Both fail `go build`.

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
// Reject: void calls and 0-value builtins as match subject
// ---------------------------------------------------------------------------

#[test]
fn void_call_match_subject_is_rejected() {
    assert_code(
        "fn noop() {}\n\nfn main() {\n    match noop() {\n        0 => {}\n        _ => {}\n    }\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn close_builtin_match_subject_is_rejected() {
    assert_code(
        "fn main() {\n    ch := make(chan int, 1)\n    match close(ch) {\n        0 => {}\n        _ => {}\n    }\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Reject: pattern literal type mismatch against subject
// ---------------------------------------------------------------------------

#[test]
fn int_pattern_against_string_subject_is_rejected() {
    assert_code(
        "fn main() {\n    s := \"hello\"\n    match s {\n        123 => {}\n        _ => {}\n    }\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn string_pattern_against_int_subject_is_rejected() {
    assert_code(
        "fn main() {\n    n := 42\n    match n {\n        \"not_an_int\" => {}\n        _ => {}\n    }\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn string_pattern_against_bool_subject_is_rejected() {
    assert_code(
        "fn main() {\n    b := true\n    match b {\n        \"yes\" => {}\n        _ => {}\n    }\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Accept: matching pattern literal types
// ---------------------------------------------------------------------------

#[test]
fn int_pattern_against_int_subject_is_accepted() {
    assert_ok(
        "fn main() {\n    n := 42\n    match n {\n        42 => {}\n        _ => {}\n    }\n}\n",
    );
}

#[test]
fn string_pattern_against_string_subject_is_accepted() {
    assert_ok(
        "fn main() {\n    s := \"hello\"\n    match s {\n        \"hello\" => {}\n        _ => {}\n    }\n}\n",
    );
}

#[test]
fn bool_pattern_against_bool_subject_is_accepted() {
    assert_ok(
        "fn main() {\n    b := true\n    match b {\n        true => {}\n        _ => {}\n    }\n}\n",
    );
}

// ---------------------------------------------------------------------------
// Accept: wildcard patterns and any subject type
// ---------------------------------------------------------------------------

#[test]
fn wildcard_patterns_against_any_subject_is_accepted() {
    assert_ok(
        "fn noop() {}\n\nfn main() {\n    match noop() {\n        _ => {}\n    }\n    match \"hello\" {\n        _ => {}\n    }\n}\n",
    );
}
