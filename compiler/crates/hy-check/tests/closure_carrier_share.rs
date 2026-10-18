//! closure_carrier_share.rs — regression test: a closure hidden inside a value
//! the checker reads as sendable must not launder the closure's captures past
//! the freeze rule.
//!
//! The channel case is the sharp one: `chan T` is sendable by type regardless
//! of T, so the shape reading cannot see the closure and the sharing walk never
//! runs. The plain-channel case is the control.

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
fn a_channel_that_carries_closures_is_not_a_plain_sendable() {
    // `made := make(chan fn(), 1)` reads as a channel (sendable by type), yet
    // every value crossing the boundary through it is a closure environment.
    let src = r#"
fn worker(c chan fn(), done chan bool) {
    _ = c
    done <- true
}

fn main() {
    done := make(chan bool, 1)
    made := make(chan fn(), 1)
    task worker(move made, done)
    <-done
}
"#;
    let got = codes(&check(src));
    assert!(
        !got.is_empty(),
        "a channel carrying closures must not pass the boundary silently, got {:?}",
        got
    );
}

#[test]
fn a_plain_channel_move_keeps_its_behaviour() {
    let src = r#"
fn worker(c chan int, done chan bool) {
    _ = c
    done <- true
}

fn main() {
    done := make(chan bool, 1)
    made := make(chan int, 1)
    task worker(move made, done)
    <-done
}
"#;
    let got = codes(&check(src));
    assert!(
        got.is_empty(),
        "a plain channel move must stay accepted, got {:?}",
        got
    );
}
