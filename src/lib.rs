extern crate libc;
extern crate bzip2;
extern crate byteorder;

#[allow(dead_code, unused_mut, non_snake_case)]
pub mod bsdiff;
pub mod patch;

use std::io::{Cursor, Write};
use std::slice;
use std::fmt::UpperHex;

#[test]
fn test_it() {
    let one = vec![1, 2, 3, 4, 6, 7, 2, 4];
    let two = vec![1];
    unsafe {
        let mut cursor = Cursor::new(Vec::new());
        let result = bsdiff::bsdiff(
            one.as_ptr(),
            one.len() as isize,
            two.as_ptr(),
            two.len() as isize,
            &mut cursor,
            test_write
        );
        assert_eq!(result, 0);
        cursor.set_position(0);

        for x in cursor.get_ref().iter().enumerate() {
            println!("{}: {:X} - a", x.0, x.1);
        }

        let mut patched = vec![0];
        let result2 = patch::bspatch(&one, &mut cursor, &mut patched).unwrap();
        assert_eq!(patched, two);
    }
}

#[cfg(test)]
unsafe extern "C" fn test_write(writer: &mut Cursor<Vec<u8>>, buffer: *const libc::c_void,
    size: libc::c_int,
) -> libc::c_int {
    match writer.write(slice::from_raw_parts(buffer as *mut u8, size as usize)) {
        Ok(x) => if x == size as usize {0} else {-1},
        Err(_) => -1 
    }
}