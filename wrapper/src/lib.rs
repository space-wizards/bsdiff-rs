extern crate libc;
extern crate bzip2;

#[allow(dead_code, unused_mut, non_snake_case)]
mod bsdiff;
#[allow(dead_code, unused_mut, non_snake_case)]
mod bspatch;

use std::io::{Write, Read, Cursor};
use std::mem;
use std::slice;
use std::ptr;

// So FFI can access both the len and buffer.
#[repr(C)]
pub struct diff_result {
    pub data: *const libc::uint8_t,
    pub length: libc::uint64_t
}

// If the diff_result's data is null, then it failed.
#[no_mangle]
pub unsafe extern "C" fn diff(
    old: *const libc::uint8_t,
    oldsize: libc::uint64_t,
    new: *const libc::uint8_t,
    newsize: libc::uint64_t,
) -> diff_result {
    let mut buffer: Cursor<Vec<u8>> = Cursor::new(Vec::new());

    let mut stream = bsdiff::bsdiff_stream {
        opaque: mem::transmute(&mut buffer), // Cast to *mut c_void
        malloc: libc::malloc,
        free: libc::free,
        write: diff_write,
    };

    // Let it diff.
    let status = bsdiff::bsdiff(old, oldsize as isize, new, newsize as isize, &mut stream as *mut bsdiff::bsdiff_stream);

    if status == -1 {
        return diff_result { data: ptr::null(), length: 0 };
    }

    // Contents will be in the buffer.
    // Shrink to fit to remove excess capacity.
    let mut buffer = buffer.into_inner();
    buffer.shrink_to_fit();
    assert!(buffer.len() == buffer.capacity());
    let ptr = buffer.as_mut_ptr();
    let len = buffer.len();
    mem::forget(buffer);

    diff_result {
        data: ptr,
        length: len as u64
    }
}

unsafe extern "C" fn diff_write(
    stream: *mut bsdiff::bsdiff_stream,
    buffer: *const libc::c_void,
    size: libc::c_int,
) -> libc::c_int {
    let mut cursor: &mut Cursor<Vec<u8>> = mem::transmute((*stream).opaque);
    match cursor.write(slice::from_raw_parts(buffer as *const u8, size as usize)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[test]
fn test_it() {
    let one = vec![1, 2, 3];
    let two = vec![1, 2, 4];
    unsafe
    {
        let result = diff(one.as_ptr(), one.len() as u64, two.as_ptr(), two.len() as u64);
        assert!(result.data != ptr::null());
    }
}