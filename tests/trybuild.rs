//! Compile-pass / compile-fail tests for `cognomen` via trybuild.
//!
//! trybuild is the standard crate for asserting compiler error text on
//! intentional UI failures.

#[test]
fn ui() {
    let t = trybuild::TestCases::new();
    t.pass("tests/ui/pass_snake.rs");
    t.pass("tests/ui/pass_kebab.rs");
    t.pass("tests/ui/pass_cases.rs");
    t.pass("tests/ui/pass_multi.rs");
    t.compile_fail("tests/ui/fail_not_enum.rs");
    t.compile_fail("tests/ui/fail_missing_attr.rs");
    t.compile_fail("tests/ui/fail_fields.rs");
    t.compile_fail("tests/ui/fail_duplicate_case.rs");
    t.compile_fail("tests/ui/fail_collision.rs");
}
