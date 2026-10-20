//! #903 — Reject local declaration initializer type mismatches, multi-assignment
//! type mismatches, and arity mismatches (HYB-E1002).
//!
//! Accepted programs must emit valid Go code that compiles cleanly with
//! `go build` (GUARANTEES.md Row 13). Currently `hyb check` fails to validate:
//!   * `var x int = "hello"` — declared type vs initializer type mismatch
//!   * `x, y = s, x` where x is int and s is string — positional type mismatch
//!   * `a, b := 1, 2, 3` — too many values
//!   * `x, y = 10` — too few values

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
// Reject: declaration initializer type mismatch
// ---------------------------------------------------------------------------

#[test]
fn var_decl_int_init_string_is_rejected() {
    // `var x int = "hello"` — declared int, initializer is string.
    assert_code(
        "fn main() {\n    var x int = \"hello\"\n    fmt.println(\"{x}\")\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn var_decl_bool_init_int_is_rejected() {
    // `let b bool = 123` — declared bool, initializer is int.
    assert_code(
        "fn main() {\n    let b bool = 123\n    fmt.println(\"{b}\")\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn var_decl_string_init_int_is_rejected() {
    // `const s string = 456` — declared string, initializer is int.
    assert_code(
        "fn main() {\n    const s string = 456\n    fmt.println(\"{s}\")\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Reject: multi-assignment positional type mismatch
// ---------------------------------------------------------------------------

#[test]
fn multi_assign_type_mismatch_is_rejected() {
    // `x, s = s, x` where x is int and s is string — both assignments mismatch.
    assert_code(
        "fn main() {\n    var x int = 1\n    var s string = \"a\"\n    x, s = s, x\n    fmt.println(\"{x} {s}\")\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Reject: declaration arity mismatch
// ---------------------------------------------------------------------------

#[test]
fn decl_too_many_values_is_rejected() {
    // `a, b := 1, 2, 3` — two names, three values.
    assert_code(
        "fn main() {\n    a, b := 1, 2, 3\n    fmt.println(\"{a} {b}\")\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn decl_too_few_values_is_rejected() {
    // `x, y := 1` — two names, one value.
    assert_code(
        "fn main() {\n    x, y := 1\n    fmt.println(\"{x} {y}\")\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Reject: assignment arity mismatch
// ---------------------------------------------------------------------------

#[test]
fn assign_too_many_values_is_rejected() {
    // `x, y = 10, 20, 30` — two targets, three values.
    assert_code(
        "fn main() {\n    var x int = 1\n    var y int = 2\n    x, y = 10, 20, 30\n    fmt.println(\"{x} {y}\")\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn assign_too_few_values_is_rejected() {
    // `x, y = 10` — two targets, one value.
    assert_code(
        "fn main() {\n    var x int = 1\n    var y int = 2\n    x, y = 10\n    fmt.println(\"{x} {y}\")\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Accept: matching types and arities
// ---------------------------------------------------------------------------

#[test]
fn var_decl_matching_type_is_accepted() {
    // `var x int = 1` — declared int, initializer is int.
    assert_ok(
        "fn main() {\n    var x int = 1\n    fmt.println(\"{x}\")\n}\n",
    );
}

#[test]
fn var_decl_matching_string_type_is_accepted() {
    // `var s string = "hello"` — declared string, initializer is string.
    assert_ok(
        "fn main() {\n    var s string = \"hello\"\n    fmt.println(\"{s}\")\n}\n",
    );
}

#[test]
fn multi_assign_matching_types_is_accepted() {
    // `x, y = y, x` where both are int — swap is fine.
    assert_ok(
        "fn main() {\n    var x int = 1\n    var y int = 2\n    x, y = y, x\n    fmt.println(\"{x} {y}\")\n}\n",
    );
}

#[test]
fn decl_matching_arity_is_accepted() {
    // `a, b := 1, 2` — two names, two values.
    assert_ok(
        "fn main() {\n    a, b := 1, 2\n    fmt.println(\"{a} {b}\")\n}\n",
    );
}

#[test]
fn assign_matching_arity_is_accepted() {
    // `x, y = 10, 20` — two targets, two values.
    assert_ok(
        "fn main() {\n    var x int = 1\n    var y int = 2\n    x, y = 10, 20\n    fmt.println(\"{x} {y}\")\n}\n",
    );
}
