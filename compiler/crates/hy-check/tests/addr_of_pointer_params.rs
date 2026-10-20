//! addr_of_pointer_params.rs — regression test for #920: passing a value to a
//! `*T` parameter (or calling a pointer-receiver method) auto-addresses it, so
//! it is a borrow even without an explicit `&`. Returning such a pointer
//! escaped the function and was accepted with 0 diagnostics.

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
fn returning_an_auto_addressed_pointer_argument_is_rejected() {
    let src = r#"
struct box {
    v: int
}

fn id_ptr(p *box) *box {
    return p
}

fn escape() *box {
    b := box{v: 1}
    return id_ptr(b)
}

fn main() {
    _ = escape()
}
"#;
    let got = codes(&check(src));
    assert!(
        !got.is_empty(),
        "returning a pointer to a local through an auto-addressed argument must be rejected, got {:?}",
        got
    );
}

#[test]
fn a_pointer_receiver_call_that_returns_the_receiver_is_rejected() {
    let src = r#"
struct box {
    v: int
}

fn (b *box) self_ptr() *box {
    return b
}

fn escape() *box {
    b := box{v: 1}
    return b.self_ptr()
}

fn main() {
    _ = escape()
}
"#;
    let got = codes(&check(src));
    assert!(
        !got.is_empty(),
        "returning the receiver pointer of a local must be rejected, got {:?}",
        got
    );
}
