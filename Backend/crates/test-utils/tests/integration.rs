//! Integration tests for test-utils

// The package is `lcc-test-utils`, so the lib target is `lcc_test_utils`.
// The previous `use test_utils as _lib;` named a crate that does not exist
// (E0432), so this test target never compiled.
use lcc_test_utils as _lib;

#[test]
fn placeholder() {
    assert!(true, "test placeholder");
}
