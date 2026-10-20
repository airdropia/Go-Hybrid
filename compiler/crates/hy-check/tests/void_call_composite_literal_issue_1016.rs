//! #1016 — Void calls and 0-value builtins in composite literals bypass
//! front-end checks (HYB-E1002).
//!
//! Void function calls (`fn noop() {}`) and 0-value Go builtins
//! (`close(ch)`, `panic(...)`, `print(...)`, `println(...)`, `delete(...)`,
//! `clear(...)`) produce no value. When used in composite literal positions
//! (struct fields, slice items, map keys/values) or unary operands, `hyb
//! check` accepts with 0 diagnostics but `go build` rejects with
//! `(no value) used as value`.

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
// Reject: void call in struct literal field
// ---------------------------------------------------------------------------

#[test]
fn void_call_in_struct_literal_field_is_rejected() {
    assert_code(
        "struct point {\n    x int\n}\n\nfn noop() {}\n\nfn main() {\n    p1 := point{x: noop()}\n    _ = p1\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn void_builtin_in_struct_literal_field_is_rejected() {
    assert_code(
        "fn main() {\n    ch := make(chan int, 1)\n    p1 := point{x: close(ch)}\n    _ = p1\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Reject: void call in slice literal item
// ---------------------------------------------------------------------------

#[test]
fn void_call_in_slice_literal_item_is_rejected() {
    assert_code(
        "fn noop() {}\n\nfn main() {\n    ch := make(chan int, 1)\n    xs := []int{noop(), close(ch)}\n    _ = xs\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Reject: void call in map literal key/value
// ---------------------------------------------------------------------------

#[test]
fn void_call_in_map_literal_key_is_rejected() {
    assert_code(
        "fn noop() {}\n\nfn main() {\n    ch := make(chan int, 1)\n    m1 := map[string]int{noop(): \"val\"}\n    _ = m1\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn void_call_in_map_literal_value_is_rejected() {
    assert_code(
        "fn noop() {}\n\nfn main() {\n    ch := make(chan int, 1)\n    m2 := map[int]string{close(ch): \"val\"}\n    _ = m2\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Reject: void call in unary operand
// ---------------------------------------------------------------------------

#[test]
fn void_call_in_unary_operand_is_rejected() {
    assert_code(
        "fn noop() {}\n\nfn main() {\n    _ = -noop()\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Accept: value-returning call in composite literal
// ---------------------------------------------------------------------------

#[test]
fn value_call_in_struct_literal_field_is_accepted() {
    assert_ok(
        "fn id(x int) int { return x }\n\nfn main() {\n    p1 := point{x: id(1)}\n    _ = p1\n}\n",
    );
}

#[test]
fn value_call_in_slice_literal_item_is_accepted() {
    assert_ok(
        "fn id(x int) int { return x }\n\nfn main() {\n    xs := []int{id(1), id(2)}\n    _ = xs\n}\n",
    );
}
