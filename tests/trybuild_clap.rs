//! Compile-pass / compile-fail tests for the `clap` feature via trybuild.

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/clap/pass_*.rs");
    t.compile_fail("tests/ui/clap/fail_*.rs");
}
