extern crate libc;
extern crate bzip2;
extern crate byteorder;

#[allow(dead_code, unused_mut, non_snake_case)]
pub mod bsdiff;
pub mod patch;

use std::io::Cursor;

#[test]
fn test_it() {
    let one = vec![1, 2, 3, 4, 6, 7, 2, 4];
    let two = vec![1];

    let mut cursor = Cursor::new(Vec::new());
    bsdiff::bsdiff(&one, &two, &mut cursor).unwrap();

    cursor.set_position(0);

    for x in cursor.get_ref().iter().enumerate() {
        println!("{}: {:X} - a", x.0, x.1);
    }

    let mut patched = vec![0];
    patch::bspatch(&one, &mut cursor, &mut patched).unwrap();
    assert_eq!(patched, two);
}
