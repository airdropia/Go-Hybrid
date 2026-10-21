//! #990 — Reject same-scope local redeclarations and short-define type
//! mutations (HYB-E1002).
//!
//! `env_define` inserted bindings without any redeclaration check, so
//! var/let/const redeclarations, parameter clobbering, and := type mutations
//! passed `hyb check` with 0 diagnostics but failed `go build`.
//!
//! Accepted programs must emit valid Go code that compiles cleanly with
//! `go build` (GUARANTEES.md Row 13).

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

fn assert_reject_e1002(src: &str) {
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

// --- Reject: same-scope var redeclaration ---

#[test]
fn var_redeclare_same_scope() {
    assert_reject_e1002(
        "package p\nfn main() {\n    var x int = 1\n    var x int = 2\n}\n",
    );
}

#[test]
fn var_redeclare_with_init() {
    assert_reject_e1002(
        "package p\nfn main() {\n    var x int = 1\n    var x int = 2\n}\n",
    );
}

// --- Reject: same-scope := redeclaration ---

#[test]
fn short_define_redeclare_same_scope() {
    assert_reject_e1002(
        "package p\nfn main() {\n    x := 1\n    x := 2\n}\n",
    );
}

#[test]
fn short_define_redeclare_type_mutation() {
    assert_reject_e1002(
        "package p\nfn main() {\n    x := 1\n    x := \"hello\"\n}\n",
    );
}

// --- Reject: var followed by := redeclaring same name ---

#[test]
fn var_then_short_define_redeclare() {
    assert_reject_e1002(
        "package p\nfn main() {\n    var x int = 1\n    x := 2\n}\n",
    );
}

// --- Reject: const redeclaration ---

#[test]
fn const_redeclare_same_scope() {
    assert_reject_e1002(
        "package p\nfn main() {\n    const x int = 1\n    const x int = 2\n}\n",
    );
}

// --- Accept: shadowing in inner scope ---

#[test]
fn shadowing_inner_scope_accepted() {
    assert_accept(
        "package p\nfn main() {\n    var x int = 1\n    {\n        var x int = 2\n    }\n}\n",
    );
}

// --- Accept: assignment (not redeclaration) ---

#[test]
fn assignment_not_redeclaration() {
    assert_accept(
        "package p\nfn main() {\n    var x int = 1\n    x = 2\n}\n",
    );
}

// --- Accept: := with at least one new variable ---

#[test]
fn short_define_with_new_var() {
    assert_accept(
        "package p\nfn main() {\n    x := 1\n    x, y := 2, 3\n}\n",
    );
}
