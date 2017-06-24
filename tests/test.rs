extern crate bsdiff;
extern crate libc;

use bsdiff::patch;
use bsdiff::diff;

use std::io::{Cursor, Read, ErrorKind};
use std::fs::File;

#[test]
fn test_it() {
    // The test files are just build artifacts I had lying around.
    // Quite large and probably *some* similarities.
    let mut one = vec![];
    let mut two = vec![];
    let mut expected = vec![];

    File::open("tests/test_1").unwrap().read_to_end(&mut one).unwrap();
    File::open("tests/test_2").unwrap().read_to_end(&mut two).unwrap();
    File::open("tests/expected_diff").unwrap().read_to_end(&mut expected).unwrap();

    let mut cursor = Cursor::new(Vec::new());
    diff::diff(&one, &two, &mut cursor).unwrap();

    assert!(&expected == cursor.get_ref());

    cursor.set_position(0);

    let mut patched = vec![0; two.len()];
    patch::patch(&one, &mut cursor, &mut patched).unwrap();
    assert!(patched == two);
}

#[test]
fn test_too_small() {
    let one = vec![1, 2, 3];
    let two = [1, 2, 3, 4];
    let mut cursor = Cursor::new(Vec::new());

    diff::diff(&one, &two, &mut cursor).unwrap();
    cursor.set_position(0);

    let mut patched = vec![0, 3];
    let error = patch::patch(&one, &mut cursor, &mut patched).unwrap_err();

    assert_eq!(error.kind(), ErrorKind::UnexpectedEof);
}