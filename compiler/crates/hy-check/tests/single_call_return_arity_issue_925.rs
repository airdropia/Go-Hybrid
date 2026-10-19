//! #925 — `return f()` bypasses return arity and void checks (HYB-E1002).
//!
//! When a return statement is a single function call whose result count
//! doesn't match the enclosing signature, `hyb check` accepts with 0
//! diagnostics but `go build` rejects:
//!   * `return pair()` (2 results) in an `int`-returning fn → "too many return values"
//!   * `return void()` (0 results) in an `int`-returning fn → "not enough return values"
//!   * `return triple()` (3 results) in a 2-result fn → "too many return values"
//!
//! Only the error-tail zero-fill pattern is legal: `return f()` where `f`
//! returns `(T)` and the enclosing fn declares `(T, error)`.

use hy_check::check_file;
use hy_syntax::parse_file;

fn codes(src: &str) -> Vec<String> {
    let ast = parse_file("test.hy", src).expect("parses");
    check_file(&ast)
        .diagnostics
        .into_iter()
        .map(|d| d.code)
        .collect()
}

fn assert_code(src: &str, want: &str) {
    let cs = codes(src);
    assert_eq!(cs, vec![want.to_string()], "for:\n{src}");
}

fn assert_ok(src: &str) {
    let cs = codes(src);
    assert!(cs.is_empty(), "expected 0 diagnostics, got {cs:?} for:\n{src}");
}

// ---------------------------------------------------------------------------
// Reject: call returns too many results for single-result enclosing fn
// ---------------------------------------------------------------------------

#[test]
fn return_pair_in_single_result_fn_is_rejected() {
    // `return pair()` where pair returns (int, int), enclosing declares int.
    assert_code(
        "fn pair() (int, int) {\n    return 1, 2\n}\n\nfn single() int {\n    return pair()\n}\n\nfn main() {\n    fmt.println(\"{single()}\")\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn return_triple_in_double_result_fn_is_rejected() {
    // `return triple()` where triple returns (int, int, int), enclosing declares (int, int).
    assert_code(
        "fn triple() (int, int, int) {\n    return 1, 2, 3\n}\n\nfn double() (int, int) {\n    return triple()\n}\n\nfn main() {\n    a, b := double()\n    fmt.println(\"{a} {b}\")\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Reject: call returns no results (void) in value-returning enclosing fn
// ---------------------------------------------------------------------------

#[test]
fn return_void_call_in_single_result_fn_is_rejected() {
    // `return void_fn()` where void_fn returns nothing, enclosing declares int.
    assert_code(
        "fn void_fn() {\n}\n\nfn single() int {\n    return void_fn()\n}\n\nfn main() {\n    fmt.println(\"{single()}\")\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Accept: call result arity matches enclosing signature exactly
// ---------------------------------------------------------------------------

#[test]
fn return_pair_in_double_result_fn_is_accepted() {
    // `return pair()` where pair returns (int, int), enclosing declares (int, int).
    assert_ok(
        "fn pair() (int, int) {\n    return 1, 2\n}\n\nfn double() (int, int) {\n    return pair()\n}\n\nfn main() {\n    a, b := double()\n    fmt.println(\"{a} {b}\")\n}\n",
    );
}

#[test]
fn return_single_in_single_result_fn_is_accepted() {
    // `return f()` where f returns int, enclosing declares int.
    assert_ok(
        "fn f() int {\n    return 1\n}\n\nfn single() int {\n    return f()\n}\n\nfn main() {\n    fmt.println(\"{single()}\")\n}\n",
    );
}

// ---------------------------------------------------------------------------
// Accept: error-tail zero-fill — `return f()` where f returns (T),
// enclosing declares (T, error). Emitter fills in nil for error.
// ---------------------------------------------------------------------------

#[test]
fn return_single_in_error_tail_fn_is_accepted() {
    // `return f()` where f returns int, enclosing declares (int, error).
    assert_ok(
        "fn f() int {\n    return 1\n}\n\nfn err_tail() (int, error) {\n    return f()\n}\n\nfn main() {\n    x, e := err_tail()\n    fmt.println(\"{x} {e}\")\n}\n",
    );
}

// ---------------------------------------------------------------------------
// Regression: call with matching arity but wrong types still caught
// ---------------------------------------------------------------------------

#[test]
fn return_call_with_wrong_result_type_is_rejected() {
    // `return f()` where f returns string, enclosing declares int.
    assert_code(
        "fn f() string {\n    return \"s\"\n}\n\nfn single() int {\n    return f()\n}\n\nfn main() {\n    fmt.println(\"{single()}\")\n}\n",
        "HYB-E1002",
    );
}
