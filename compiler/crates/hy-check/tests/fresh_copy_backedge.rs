//! fresh_copy_backedge.rs — regression test for #372: a fresh-copy call in a
//! loop body used to clear the loop-carried move state of the caller's
//! argument root. `chunk := chunk_of(words)` made `check_loop_backedge` treat
//! `words` as a loop-local fresh-copy root (through alias resolution), so the
//! `move words` in the loop body never reported E3003 and two tasks mutated the
//! same backing array. A locally shadowed callee name made the fresh-copy
//! summary describe a function the call never reaches.

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

const FRESH_APPEND: &str = r#"
fn chunk_of(src []int) []int {
    out := []int{}
    out = append(out, src[0])
    return out
}
"#;

#[test]
fn moving_the_argument_of_a_fresh_copy_call_each_iteration_is_rejected() {
    let src = format!(
        r#"{FRESH_APPEND}
fn worker(shared []int, done chan int) {{
    shared[0] = shared[0] + 1
    done <- shared[0]
}}

fn main() {{
    done := make(chan int, 4)
    words := []int{{1, 2, 3}}
    for i := 0; i < 2; i = i + 1 {{
        chunk := chunk_of(words)
        if len(chunk) == 0 {{
            return
        }}
        task worker(move words, done)
    }}
    <-done
    <-done
}}
"#
    );
    let got = codes(&check(&src));
    assert!(
        got.iter().any(|c| c == "HYB-E3003"),
        "the loop moves `words` without reinitializing it, expected E3003, got {:?}",
        got
    );
}

#[test]
fn a_shadowed_callee_name_is_not_a_fresh_copy() {
    // `chunk_of` here is a local closure, not the package function whose body
    // the summary inspects. The call must not be treated as a fresh copy, so
    // the loop-carried move of `words` stays visible.
    let src = format!(
        r#"{FRESH_APPEND}
fn main() {{
    words := []int{{1, 2, 3}}
    chunk_of := fn(src []int) []int {{ return src }}
    for i := 0; i < 2; i = i + 1 {{
        chunk := chunk_of(words)
        if len(chunk) == 0 {{
            return
        }}
        _ = chunk
    }}
}}
"#
    );
    let got = codes(&check(&src));
    assert!(
        !got.iter().any(|c| c == "HYB-E3003"),
        "no move happens here, so E3003 must not fire; got {:?}",
        got
    );
}
