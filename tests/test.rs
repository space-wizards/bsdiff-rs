use bsdiff::patch;
use bsdiff::diff;
use std::io::ErrorKind;

#[test]
fn test_it() {
    // The test files are just build artifacts I had lying around.
    // Quite large and probably *some* similarities.
    let one = std::fs::read("tests/test_1").unwrap();
    let two = std::fs::read("tests/test_2").unwrap();
    let expected = std::fs::read("tests/expected_diff").unwrap();

    let mut patch = Vec::with_capacity(expected.len());
    diff::diff(&one, &two, &mut patch).unwrap();

    assert_eq!(&expected, &patch);

    let mut patched = Vec::with_capacity(two.len());
    patch::patch(&one, &mut patch.as_slice(), &mut patched).unwrap();
    assert!(patched == two);
}

#[test]
fn test_truncated_patch() {
    let one = vec![1, 2, 3];
    let two = [1, 2, 3, 4];
    let mut buf = Vec::new();

    diff::diff(&one, &two, &mut buf).unwrap();

    let mut patched = Vec::new();
    while buf.len() > 1 {
        buf.pop();
        let error = patch::patch(&one, &mut buf.as_slice(), &mut patched).unwrap_err();
        assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
    }
}
