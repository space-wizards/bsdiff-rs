extern crate bsdiff;
extern crate libc;

use bsdiff::patch;
use bsdiff::diff;

use std::slice;
use std::io::{Cursor, Write, Read};
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
    diff::diff(&one, &two, &mut cursor, test_write).unwrap();

    assert!(&expected == cursor.get_ref());

    cursor.set_position(0);

    let mut patched = vec![0; two.len()];
    patch::patch(&one, &mut cursor, &mut patched).unwrap();
    assert!(patched == two);
}

unsafe extern "C" fn test_write(writer: &mut Cursor<Vec<u8>>, buffer: *const libc::c_void,
    size: libc::c_int,
) -> libc::c_int {
    match writer.write(slice::from_raw_parts(buffer as *mut u8, size as usize)) {
        Ok(x) => if x == size as usize {0} else {-1},
        Err(_) => -1 
    }
}