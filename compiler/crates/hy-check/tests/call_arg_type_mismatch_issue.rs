//! Call argument type mismatch — check argument types against parameter types
//! (HYB-E1002).
//!
//! Go rejects function call arguments whose types don't match the declared
//! parameter types at compile time. Hybrid must too (GUARANTEES Row 13).

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
// Reject: call argument type mismatch
// ---------------------------------------------------------------------------

#[test]
fn call_arg_string_to_int_param_is_rejected() {
    // `foo("hello", 1)` where foo expects (int, string) — first arg mismatches.
    assert_code(
        "fn foo(a int, b string) {}\n\nfn main() {\n    foo(\"hello\", 1)\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn call_arg_int_to_string_param_is_rejected() {
    // `foo(1, "hello")` where foo expects (string, int) — first arg mismatches.
    assert_code(
        "fn foo(a string, b int) {}\n\nfn main() {\n    foo(1, \"hello\")\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn call_arg_float_to_int_param_is_rejected() {
    // `bar(2.5)` where bar expects int — float mismatches int.
    assert_code(
        "fn bar(a int) {}\n\nfn main() {\n    bar(2.5)\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn call_arg_int_to_float_param_is_rejected() {
    // `baz(1)` where baz expects float64 — int mismatches float.
    assert_code(
        "fn baz(a float64) {}\n\nfn main() {\n    baz(1)\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Accept: matching argument types
// ---------------------------------------------------------------------------

#[test]
fn call_arg_matching_types_is_accepted() {
    // `foo(1, "hello")` where foo expects (int, string) — both match.
    assert_ok(
        "fn foo(a int, b string) {}\n\nfn main() {\n    foo(1, \"hello\")\n}\n",
    );
}

#[test]
fn call_arg_int_to_int_param_is_accepted() {
    // `bar(1)` where bar expects int — matches.
    assert_ok(
        "fn bar(a int) {}\n\nfn main() {\n    bar(1)\n}\n",
    );
}

#[test]
fn call_arg_float_to_float_param_is_accepted() {
    // `baz(2.5)` where baz expects float64 — matches.
    assert_ok(
        "fn baz(a float64) {}\n\nfn main() {\n    baz(2.5)\n}\n",
    );
}
