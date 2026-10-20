//! #1023 — Void calls and 0-value builtins in field/method selection bases
//! bypass front-end checks (HYB-E1002).
//!
//! Void function calls (`fn noop() {}`) and 0-value Go builtins
//! (`close(ch)`, `panic(...)`, `print(...)`, `println(...)`, `delete(...)`,
//! `clear(...)`) produce no value. When used as the base of a field selection
//! or method call, `hyb check` accepts with 0 diagnostics but `go build`
//! rejects with `(no value) used as value`.

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
// Reject: void call as field selection base
// ---------------------------------------------------------------------------

#[test]
fn void_call_field_selection_base_is_rejected() {
    assert_code(
        "struct point {\n    x int\n}\n\nfn noop() {}\n\nfn main() {\n    _ = noop().x\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Reject: void call as method call receiver
// ---------------------------------------------------------------------------

#[test]
fn void_call_method_receiver_is_rejected() {
    assert_code(
        "fn noop() {}\n\nfn main() {\n    noop().run()\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Reject: 0-value builtin as field selection base
// ---------------------------------------------------------------------------

#[test]
fn close_builtin_field_selection_base_is_rejected() {
    assert_code(
        "struct point {\n    x int\n}\n\nfn main() {\n    ch := make(chan int, 1)\n    _ = close(ch).x\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn panic_builtin_method_receiver_is_rejected() {
    assert_code(
        "fn main() {\n    panic(\"abort\").run()\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Accept: value-returning call as field selection base
// ---------------------------------------------------------------------------

#[test]
fn value_call_field_selection_base_is_accepted() {
    assert_ok(
        "fn id(x int) int { return x }\n\nstruct point { x int }\n\nfn main() {\n    p := point{ x: id(1) }\n    _ = p.x\n}\n",
    );
}
