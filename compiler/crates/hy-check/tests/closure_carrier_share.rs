//! closure_carrier_share.rs — regression test: a closure hidden inside a value
//! that the checker reads as sendable-by-type (a channel of closures) must not
//! launder the closure's captures past the freeze rule. The shape reading says
//! `chan T` is sendable regardless of T, so the moved value has to be inspected
//! directly.

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
fn moving_a_composite_that_embeds_a_closure_records_a_share_edge() {
    // `box{fn: get_m}` holds a closure whose body reads `m`. Moving the box
    // into a task must not leave `m` writable from this scope.
    let src = r#"
struct box {
    fn: fn() map[string]int
}

fn worker(b box, done chan bool) {
    _ = b
    done <- true
}

fn main() {
    done := make(chan bool, 1)
    m := map[string]int{"k": 0}
    get_m := fn() map[string]int { return m }
    b := box{fn: get_m}
    task worker(move b, done)
    m["k"] = 2
    <-done
}
"#;
    let got = codes(&check(src));
    assert!(
        !got.is_empty(),
        "moving a box that embeds a closure must produce a diagnostic, got {:?}",
        got
    );
}

#[test]
fn a_plain_composite_without_closures_keeps_its_behaviour() {
    let src = r#"
struct point {
    x: int
}

fn worker(p point, done chan bool) {
    _ = p
    done <- true
}

fn main() {
    done := make(chan bool, 1)
    p := point{x: 1}
    task worker(move p, done)
    <-done
}
"#;
    let got = codes(&check(src));
    assert!(
        got.is_empty(),
        "a value-only struct move must stay accepted, got {:?}",
        got
    );
}
