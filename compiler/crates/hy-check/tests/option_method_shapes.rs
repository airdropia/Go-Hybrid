//! option_method_shapes.rs — regression test for #884 part 2: the T20 prelude's
//! `option[T]` methods must carry the payload shape through. `.unwrap()` on an
//! option holding a map returns that map's shape, so a copy of the unwrapped
//! value still records a sharing edge and the freeze rule keeps working.

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
fn unwrapping_an_option_keeps_the_payload_sharing_edge() {
    let src = r#"
fn map_worker(m map[string]int, ch chan map[string]int) {
    m["w"] = 1
    ch <- m
}

fn main() {
    ch := make(chan map[string]int, 1)
    m := map[string]int{"k": 0}
    o := some(m)
    m2 := o.unwrap()
    m3 := m2
    task map_worker(move m2, ch)
    m3["main"] = 2
}
"#;
    let got = codes(&check(src));
    assert!(
        !got.is_empty(),
        "an unwrapped map must keep its sharing edge, got {:?}",
        got
    );
}

#[test]
fn or_default_returns_the_payload_shape() {
    let src = r#"
fn worker(m map[string]int, done chan bool) {
    m["w"] = 1
    done <- true
}

fn main() {
    done := make(chan bool, 1)
    m := map[string]int{"k": 0}
    o := some(m)
    m2 := o.or(m)
    task worker(move m, done)
    m2["main"] = 2
    <-done
}
"#;
    let got = codes(&check(src));
    assert!(
        !got.is_empty(),
        "or() must carry the payload shape, got {:?}",
        got
    );
}
