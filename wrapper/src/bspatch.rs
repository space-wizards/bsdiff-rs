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

#[derive(Copy)]
#[repr(C)]
pub struct bspatch_stream {
    pub opaque : *mut ::std::os::raw::c_void,
    pub read : unsafe extern fn(*const bspatch_stream, *mut ::std::os::raw::c_void, i32) -> i32,
}

impl Clone for bspatch_stream {
    fn clone(&self) -> Self { *self }
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

pub unsafe fn bspatch(
    mut old : *const u8,
    mut oldsize : isize,
    mut new : *mut u8,
    mut newsize : isize,
    mut stream : *mut bspatch_stream
) -> i32 {
    let mut _currentBlock;
    let mut buf : [u8; 8] = [0; 8];
    let mut oldpos : isize;
    let mut newpos : isize;
    let mut ctrl : [isize; 3] = [0; 3];
    let mut i : isize;
    oldpos = 0isize;
    newpos = 0isize;
    'loop1: loop {
        if !(newpos < newsize) {
            _currentBlock = 2;
            break;
        }
        i = 0isize;
        'loop4: loop {
            if !(i <= 2isize) {
                break;
            }
            if ((*stream).read)(
                   stream as (*const bspatch_stream),
                   buf.as_mut_ptr() as (*mut ::std::os::raw::c_void),
                   8i32
               ) != 0 {
                _currentBlock = 22;
                break 'loop1;
            }
            ctrl[i as (usize)] = offtin(buf.as_mut_ptr());
            i = i + 1isize;
        }
        if newpos + ctrl[0usize] > newsize {
            _currentBlock = 18;
            break;
        }
        if ((*stream).read)(
               stream as (*const bspatch_stream),
               new.offset(newpos) as (*mut ::std::os::raw::c_void),
               ctrl[0usize] as (i32)
           ) != 0 {
            _currentBlock = 17;
            break;
        }
        i = 0isize;
        'loop8: loop {
            if !(i < ctrl[0usize]) {
                break;
            }
            if oldpos + i >= 0isize && (oldpos + i < oldsize) {
                let _rhs = *old.offset(oldpos + i);
                let _lhs = &mut *new.offset(newpos + i);
                *_lhs = (*_lhs as (i32) + _rhs as (i32)) as (u8);
            }
            i = i + 1isize;
        }
        newpos = newpos + ctrl[0usize];
        oldpos = oldpos + ctrl[0usize];
        if newpos + ctrl[1usize] > newsize {
            _currentBlock = 13;
            break;
        }
        if ((*stream).read)(
               stream as (*const bspatch_stream),
               new.offset(newpos) as (*mut ::std::os::raw::c_void),
               ctrl[1usize] as (i32)
           ) != 0 {
            _currentBlock = 12;
            break;
        }
        newpos = newpos + ctrl[1usize];
        oldpos = oldpos + ctrl[2usize];
    }
    if _currentBlock == 2 {
        0i32
    } else if _currentBlock == 12 {
        -1i32
    } else if _currentBlock == 13 {
        -1i32
    } else if _currentBlock == 17 {
        -1i32
    } else if _currentBlock == 18 {
        -1i32
    } else {
        -1i32
    }
}
