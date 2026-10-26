// ============================================================================
// inline-closure-arity-reject — Checker corpus (bounty track: "grow the
// adversarial corpus").
//
// PURPOSE: an inline closure call that passes the wrong number of arguments
// or wrong argument types. The emitted Go call cannot build, so accepting it
// breaks GUARANTEES.md Row 13 (BUILDABLE).
//
// RULE CITATION: Row 13 — every `hyb check`-accepted file builds cleanly with
// `go build`. Go refuses both directions at the call site:
//   `not enough arguments in call to <closure>`
//   `too many arguments in call to <closure>`
//   `cannot use ... as int value in argument to <closure>`
//
// ---- Why this is a REJECT ----
// An inline closure `fn(x int) {}` declares one int parameter. Calling it with
// `(1, 2)` (two args) or `("hello")` (wrong type) passes `hyb check` with 0
// diagnostics but the emitted Go fails to build. Closed by the issue #1877
// fix (`buildable.rs::check_call_operands` Expr::FnLit arm +
// `buildable.rs::Stmt::Task` Expr::FnLit arm).
//
// ---- Expected verdict (reproduced as a real `go build` failure first) ----
// EXPECTED-VERDICT: REJECT BUILDABLE
// EXPECTED-DIAGNOSTIC: HYB-E1002
//
// ---- Paired control ----
// `call-and-return-controls-accept.hy` holds the same shapes with the right
// argument counts and types, so this rejection cannot be widened into a
// blanket rule.
//
// ---- Annotations / managed ----
//   lifetime annotations: 0. `managed` blocks: 0.
// ============================================================================

fn main() {
    // Wrong arity: closure takes 1 arg, called with 2
    (fn(x int) int {
        return x
    })(1, 2)

    // Wrong type: closure takes int, called with string
    (fn(y int) int {
        return y
    })("hello")
}
