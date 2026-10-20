//! composite_binding_sendable.rs — regression test for #723: a value composite
//! (struct of value fields, or a fixed array) is accepted when written as a
//! literal and must be accepted when bound to a variable first. Composites that
//! carry reference storage keep the O4 rejection.

use hy_check::check_file;
use hy_diag::Severity;
use hy_syntax::parse_file;

fn check(src: &str) -> hy_check::CheckResult {
    let path = "inline.hy";
    let ast = parse_file(path, src).unwrap_or_else(|diags| panic!("must parse: {:?}", diags));
    check_file(&ast)
}

fn codes(r: &hy_check::CheckResult) -> Vec<String> {
    r.diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.clone())
        .collect()
}

#[test]
fn a_value_only_struct_binding_is_sendable_like_its_literal() {
    let src = r#"
struct pair {
    a: int
    b: int
}

fn worker(p pair, done chan bool) {
    _ = p
    done <- true
}

fn main() {
    done := make(chan bool, 1)
    var p pair
    p.a = 1
    task worker(p, done)
    p.a = 9
    <-done
}
"#;
    let got = codes(&check(src));
    assert!(
        got.is_empty(),
        "a value-only struct passed by value must be sendable, got {:?}",
        got
    );
}

#[test]
fn a_struct_holding_a_map_is_still_rejected() {
    let src = r#"
struct holder {
    m map[string]int
}

fn worker(h holder, done chan bool) {
    h.m["k"] = 1
    done <- true
}

fn main() {
    done := make(chan bool, 1)
    var h holder
    h.m = map[string]int{"k": 0}
    task worker(h, done)
    <-done
}
"#;
    let got = codes(&check(src));
    assert!(
        !got.is_empty(),
        "a struct that carries reference storage must keep its rejection, got {:?}",
        got
    );
}
