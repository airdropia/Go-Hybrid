//! Return type mismatch — check return value types against declared result
//! types (HYB-E1002).
//!
//! Go rejects return statements whose value types don't match the declared
//! result types at compile time. Hybrid must too (GUARANTEES Row 13).

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
// Reject: return type mismatch
// ---------------------------------------------------------------------------

#[test]
fn return_string_to_int_param_is_rejected() {
    // `return "hello"` where fn declares int result — string mismatches int.
    assert_code(
        "fn foo() int {\n    return \"hello\"\n}\n\nfn main() {\n    _ = foo()\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn return_int_to_string_param_is_rejected() {
    // `return 1` where fn declares string result — int mismatches string.
    assert_code(
        "fn foo() string {\n    return 1\n}\n\nfn main() {\n    _ = foo()\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn return_float_to_int_param_is_rejected() {
    // `return 2.5` where fn declares int result — float mismatches int.
    assert_code(
        "fn foo() int {\n    return 2.5\n}\n\nfn main() {\n    _ = foo()\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn return_int_to_float_param_is_rejected() {
    // `return 1` where fn declares float64 result — int mismatches float.
    assert_code(
        "fn foo() float64 {\n    return 1\n}\n\nfn main() {\n    _ = foo()\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Accept: matching return types
// ---------------------------------------------------------------------------

#[test]
fn return_matching_type_is_accepted() {
    // `return 1` where fn declares int result — matches.
    assert_ok(
        "fn foo() int {\n    return 1\n}\n\nfn main() {\n    _ = foo()\n}\n",
    );
}

#[test]
fn return_matching_string_type_is_accepted() {
    // `return "hello"` where fn declares string result — matches.
    assert_ok(
        "fn foo() string {\n    return \"hello\"\n}\n\nfn main() {\n    _ = foo()\n}\n",
    );
}

#[test]
fn return_matching_float_type_is_accepted() {
    // `return 2.5` where fn declares float64 result — matches.
    assert_ok(
        "fn foo() float64 {\n    return 2.5\n}\n\nfn main() {\n    _ = foo()\n}\n",
    );
}
