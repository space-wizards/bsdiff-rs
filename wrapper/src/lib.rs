extern crate libc;
extern crate bzip2;
extern crate byteorder;

#[allow(dead_code, unused_mut, non_snake_case)]
mod bsdiff;
#[allow(dead_code, unused_mut, non_snake_case)]
mod bspatch;

use std::io::{Write, Read, Cursor};
use std::mem;
use std::slice;
use std::ptr;
use bzip2::write::BzEncoder;
use bzip2::read::BzDecoder;
use byteorder::{NetworkEndian, WriteBytesExt, ByteOrder};

// So FFI can access both the len and buffer.
#[repr(C)]
pub struct diff_result {
    pub data: *mut libc::uint8_t,
    pub length: libc::uint64_t,
}

// If the diff_result's data is null, then it failed.
// The diff result MUST be reclaimed with diff_cleanup to prevent leaks.
#[no_mangle]
pub unsafe extern "C" fn bzip2_diff(
    old: *const libc::uint8_t,
    oldsize: libc::uint64_t,
    new: *const libc::uint8_t,
    newsize: libc::uint64_t,
) -> diff_result {
    let mut buffer = BzEncoder::new(Cursor::new(Vec::new()), bzip2::Compression::Default);

    // Let it diff.
    let status = bsdiff::bsdiff(
        old,
        oldsize as isize,
        new,
        newsize as isize,
        &mut buffer,
        libc::malloc,
        libc::free,
        diff_write,
    );

    if status == -1 {
        return diff_result {
            data: ptr::null_mut(),
            length: 0,
        };
    }

    // Contents will be in the buffer.
    // Write len out to raw stream.
    let mut buffer = buffer.finish().unwrap();
    buffer.write_u64::<NetworkEndian>(newsize).unwrap();

    // Take vec out and hand data back to C#.
    let mut buffer = buffer.into_inner();
    buffer.shrink_to_fit();
    assert!(buffer.len() == buffer.capacity());
    let ptr = buffer.as_mut_ptr();
    let len = buffer.len();

    // Kill vec so that the pointer remains allocated and C# can read it.
    mem::forget(buffer);

    diff_result {
        data: ptr,
        length: len as u64,
    }
}

unsafe extern "C" fn diff_write(
    stream: &mut BzEncoder<Cursor<Vec<u8>>>,
    buffer: *const libc::c_void,
    size: libc::c_int,
) -> libc::c_int {
    println!("{}, {:?}", size, &slice::from_raw_parts(buffer as *mut u8, size as usize));
    match stream.write(slice::from_raw_parts(buffer as *const u8, size as usize)) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[no_mangle]
pub unsafe extern "C" fn diff_cleanup(data: diff_result) {
    let diff_result {
        data: ptr,
        length: len,
    } = data;
    // Allocate into a vec and drop it, so the vec clears it out.
    let vec = Vec::from_raw_parts(ptr, len as usize, len as usize);
    // Drop call to be nice and ~explicit~.
    mem::drop(vec);
}


#[no_mangle]
pub unsafe extern "C" fn bzip2_patch(
    old: *const libc::uint8_t,
    oldsize: libc::uint64_t,
    patch: *const libc::uint8_t,
    patchsize: libc::uint64_t,
) -> diff_result {
    let new_slice = slice::from_raw_parts(patch, patchsize as usize);
    // Pop off size footer.
    let len = NetworkEndian::read_u64(&new_slice[new_slice.len() - 8..new_slice.len()]);

    let mut stream = BzDecoder::new(Cursor::new(&new_slice[..new_slice.len() - 8]));
    let mut output_buffer = vec![0; len as usize];

    let result = bspatch::bspatch(
        old,
        oldsize as i64,
        output_buffer.as_ptr() as *mut u8,
        len as i64,
        &mut stream,
        diff_read,
    );

    if result == -1 {
        return diff_result {
            data: ptr::null_mut(),
            length: 0,
        };
    }

    output_buffer.shrink_to_fit();
    assert!(output_buffer.len() == output_buffer.capacity());
    let ptr = output_buffer.as_mut_ptr();
    let len = output_buffer.len();

    // Kill vec so that the pointer remains allocated and C# can read it.
    mem::forget(output_buffer);

    diff_result {
        data: ptr,
        length: len as u64,
    }
}

unsafe extern "C" fn diff_read(
    stream: &mut BzDecoder<Cursor<&[u8]>>,
    buffer: *mut libc::c_void,
    size: libc::c_int,
) -> libc::c_int {
    let mut out_slice = slice::from_raw_parts_mut(buffer as *mut u8, size as usize);
    match stream.read(out_slice) {
        Ok(len) => if len == size as usize { 0 } else { -1 },
        Err(a) => {
            println!("{}", a);
            -1
        }
    }
}

#[test]
fn test_it() {
    let one = vec![1, 2, 3, 4, 6, 7, 2, 4];
    let two = vec![1];
    unsafe {
        let result = bzip2_diff(
            one.as_ptr(),
            one.len() as u64,
            two.as_ptr(),
            two.len() as u64,
        );
        println!("{:X}, {}", result.data as usize, result.length);
        assert!(result.data != ptr::null_mut());
        let result2 = bzip2_patch(one.as_ptr(), one.len() as u64, result.data, result.length);
        assert!(result2.data != ptr::null_mut());
        let AAAAH = slice::from_raw_parts(result2.data, result2.length as usize).to_owned();
        println!("{:?}:{:?}", AAAAH, two);
        assert!(AAAAH == two);
        diff_cleanup(result);
    }
}
