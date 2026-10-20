//! field_slot_loan.rs — regression test for #831: a field-projected slot loan
//! (`&m["a"].sub`) must be recorded against the indexed base path, so the
//! borrow cannot outlive a rebind of that path.

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
fn rebinding_the_indexed_base_of_a_field_projected_loan_is_rejected() {
    let src = r#"
struct item {
    x int
}

fn worker(m map[string]item, done chan bool) {
    m["a"] = item{x: 1}
    done <- true
}

fn main() {
    m := map[string]item{"a": item{x: 1}}
    done := make(chan bool, 1)
    n := &m["a"].x
    m = map[string]item{"a": item{x: 2}}
    task worker(move m, done)
    n = 99
    <-done
}
"#;
    let got = codes(&check(src));
    assert!(
        !got.is_empty(),
        "a rebind of the indexed base must invalidate the field-projected loan, got {:?}",
        got
    );
}
