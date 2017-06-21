/*-
 * Copyright 2003-2005 Colin Percival
 * Copyright 2012 Matthew Endsley
 * All rights reserved
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted providing that the following conditions 
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 *
 * THIS SOFTWARE IS PROVIDED BY THE AUTHOR ``AS IS'' AND ANY EXPRESS OR
 * IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED
 * WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE
 * ARE DISCLAIMED.  IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR ANY
 * DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
 * DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS
 * OR SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
 * STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING
 * IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED OF THE
 * POSSIBILITY OF SUCH DAMAGE.
 */

use libc;

pub unsafe fn bspatch<T>(
    mut old : *const u8,
    mut oldsize : i64,
    mut new : *mut u8,
    mut newsize : i64,
    mut opaque : &mut T,
    read: unsafe extern fn(&mut T, *mut libc::c_void, i32) -> i32
) -> i32 {
    let mut buf = [0u8; 8];
    let mut oldpos = 0i64;
    let mut newpos = 0i64;
    let mut ctrl = [0i64; 3];

    while newpos < newsize {
        println!("{}, {}", newpos, newsize);
        // Read control data
        for i in 0..3 {
            if read(opaque, buf.as_mut_ptr() as *mut libc::c_void, 8) != 0 {
                return -1;
            }
            ctrl[i] = offtin(buf.as_mut_ptr()) as i64;
        }

        // Sanity-check
        if newpos + ctrl[0] > newsize {
            return -1;
        }

        // Read diff string
        if read(opaque, new.offset(newpos as isize) as *mut libc::c_void, ctrl[0] as i32) != 0 {
            return -1;
        }

        // Add old data to diff string
        for i in 0..ctrl[0] {
            if oldpos+1 > 0 && oldpos+i < oldsize {
                *(new.offset(newpos as isize +i as isize)) += *(old.offset(oldpos as isize + i as isize));
            }
        }

        // Adjust pointers
        newpos += ctrl[0];
        oldpos += ctrl[0];

        // Sanity-check
        if newpos + ctrl[1] > newsize {
            return -1;
        }

        // Read extra string
        if read(opaque, new.offset(newpos as isize) as *mut libc::c_void, ctrl[1] as i32) != 0 {
            return -1;
        }

        // Adjust pointers
        newpos += ctrl[1];
        oldpos += ctrl[2];
    }

    0
}


unsafe fn offtin(mut buf : *mut u8) -> isize {
    let mut y : isize;
    y = (*buf.offset(7isize) as (i32) & 0x7fi32) as (isize);
    y = y * 256isize;
    y = y + *buf.offset(6isize) as (isize);
    y = y * 256isize;
    y = y + *buf.offset(5isize) as (isize);
    y = y * 256isize;
    y = y + *buf.offset(4isize) as (isize);
    y = y * 256isize;
    y = y + *buf.offset(3isize) as (isize);
    y = y * 256isize;
    y = y + *buf.offset(2isize) as (isize);
    y = y * 256isize;
    y = y + *buf.offset(1isize) as (isize);
    y = y * 256isize;
    y = y + *buf.offset(0isize) as (isize);
    if *buf.offset(7isize) as (i32) & 0x80i32 != 0 {
        y = -y;
    }
    y
}
