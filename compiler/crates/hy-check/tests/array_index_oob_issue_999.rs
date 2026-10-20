//! #999 — const index OOB via assignment on var-declared array (HYB-E1002).
//!
//! `var a [2]int; a[300] = 1` passes `hyb check` with 0 diagnostics but
//! `go build` fails with `index 300 out of bounds [0:2]`. The array-literal
//! form (#910) is already checked, but the var-declared array assignment
//! target fell through.

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
// Reject: constant index out of bounds on array assignment target
// ---------------------------------------------------------------------------

#[test]
fn var_array_oob_index_assign_is_rejected() {
    // `var a [2]int; a[300] = 1` — index 300 >= array size 2.
    assert_code(
        "fn main() {\n    var a [2]int\n    a[300] = 1\n    fmt.println(\"{a[0]}\")\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn var_array_oob_index_assign_edge_is_rejected() {
    // `var a [2]int; a[2] = 1` — index 2 == array size 2 (OOB).
    assert_code(
        "fn main() {\n    var a [2]int\n    a[2] = 1\n}\n",
        "HYB-E1002",
    );
}

#[test]
fn var_array_large_oob_index_assign_is_rejected() {
    // `var a [3]int; a[999] = 1` — index 999 >= array size 3.
    assert_code(
        "fn main() {\n    var a [3]int\n    a[999] = 1\n}\n",
        "HYB-E1002",
    );
}

// ---------------------------------------------------------------------------
// Accept: in-bounds index on array assignment target
// ---------------------------------------------------------------------------

#[test]
fn var_array_in_bounds_index_assign_is_accepted() {
    // `var a [2]int; a[0] = 1` and `a[1] = 1` — both valid.
    assert_ok(
        "fn main() {\n    var a [2]int\n    a[0] = 1\n    a[1] = 2\n    fmt.println(\"{a[0]} {a[1]}\")\n}\n",
    );
}

#[test]
fn var_array_last_index_assign_is_accepted() {
    // `var a [2]int; a[1] = 1` — last valid index.
    assert_ok(
        "fn main() {\n    var a [2]int\n    a[1] = 1\n}\n",
    );
}

// ---------------------------------------------------------------------------
// Accept: non-constant index stays fail-open (cannot check at compile time)
// ---------------------------------------------------------------------------

#[test]
fn var_array_variable_index_assign_is_accepted() {
    // `var a [2]int; var i int; a[i] = 1` — index is a variable, fail-open.
    assert_ok(
        "fn main() {\n    var a [2]int\n    var i int\n    i = 0\n    a[i] = 1\n}\n",
    );
}

// ---------------------------------------------------------------------------
// Accept: non-array base stays silent (no type info)
// ---------------------------------------------------------------------------

#[test]
fn var_slice_oob_index_assign_is_accepted() {
    // Slices have dynamic length — cannot check at compile time.
    assert_ok(
        "fn main() {\n    var s []int\n    s[300] = 1\n}\n",
    );
}
