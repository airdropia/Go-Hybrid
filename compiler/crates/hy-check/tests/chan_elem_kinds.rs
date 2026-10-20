//! chan_elem_kinds.rs — regression test for #887: a channel send must agree
//! with the channel's element kind. `ch <- true` / `ch <- 1.5` on a `chan int`
//! used to pass `hyb check` and then fail `go build`.

use hy_check::check_file;
use hy_diag::Severity;
use hy_syntax::parse_file;

fn check(src: &str) -> hy_check::CheckResult {
    let path = "inline.hy";
    let ast = parse_file(path, src).unwrap_or_else(|diags| panic!("must parse: {:?}", diags));
    check_file(&ast)
}

fn errors(r: &hy_check::CheckResult) -> Vec<String> {
    r.diagnostics
        .iter()
        .filter(|d| d.severity == Severity::Error)
        .map(|d| d.code.clone())
        .collect()
}

#[test]
fn sending_a_bool_on_an_int_channel_is_rejected() {
    let src = r#"
fn main() {
    ch := make(chan int, 1)
    ch <- true
}
"#;
    let got = errors(&check(src));
    assert!(
        !got.is_empty(),
        "a bool cannot be sent on a chan int, got {:?}",
        got
    );
}

#[test]
fn sending_a_matching_kind_is_accepted() {
    let src = r#"
fn main() {
    ch := make(chan int, 1)
    v := 1
    ch <- v
}
"#;
    let got = errors(&check(src));
    assert!(got.is_empty(), "a matching send must stay accepted, got {:?}", got);
}
