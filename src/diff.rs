/*-
 * Copyright 2003-2005 Colin Percival
 * Copyright 2012 Matthew Endsley
 * Modified 2017 Pieter-Jan Briers
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

use std::ptr;
use std::io;
use std::io::Write;
use std::mem::size_of;
use libc;

pub fn diff<T>(mut old: &[u8],
                 mut new: &[u8],
                 mut writer: &mut T,
                 write: unsafe extern "C" fn(&mut T, *const libc::c_void, i32) -> i32)
                 -> io::Result<()>
    where T: Write
{
    unsafe {

        let I = libc::malloc((old.len() + 1).wrapping_mul(size_of::<isize>())) as *mut isize;
        if I == ptr::null_mut() {
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to malloc I."));
        }

        let buffer = libc::malloc(new.len() + 1) as *mut u8;
        if buffer == ptr::null_mut() {
            libc::free(I as *mut libc::c_void);
            return Err(io::Error::new(io::ErrorKind::Other, "Failed to malloc buffer."));
        }

        let mut req: bsdiff_request = bsdiff_request {
            old: old.as_ptr(),
            oldsize: old.len() as isize,
            new: new.as_ptr(),
            newsize: new.len() as isize,
            I: I,
            buffer: buffer,
        };

        let result = bsdiff_internal(&mut req, writer, write);
        libc::free(req.buffer as (*mut libc::c_void));
        libc::free(req.I as (*mut libc::c_void));

        match result {
            0 => Ok(()),
            _ => Err(io::Error::new(io::ErrorKind::Other, "Meh")),
        }
    }
}


extern "C" {
    fn memcmp(__s1: *const libc::c_void, __s2: *const libc::c_void, __n: usize) -> i32;
}

unsafe fn split(mut I: *mut isize,
                mut V: *mut isize,
                mut start: isize,
                mut len: isize,
                mut h: isize) {
    let mut i: isize;
    let mut j: isize;
    let mut k: isize;
    let mut x: isize;
    let mut tmp: isize;
    let mut jj: isize;
    let mut kk: isize;
    if len < 16isize {
        k = start;
        'loop31: loop {
            if !(k < start + len) {
                break;
            }
            j = 1isize;
            x = *V.offset(*I.offset(k) + h);
            i = 1isize;
            'loop34: loop {
                if !(k + i < start + len) {
                    break;
                }
                if *V.offset(*I.offset(k + i) + h) < x {
                    x = *V.offset(*I.offset(k + i) + h);
                    j = 0isize;
                }
                if *V.offset(*I.offset(k + i) + h) == x {
                    tmp = *I.offset(k + j);
                    *I.offset(k + j) = *I.offset(k + i);
                    *I.offset(k + i) = tmp;
                    j = j + 1isize;
                }
                i = i + 1isize;
            }
            i = 0isize;
            'loop36: loop {
                if !(i < j) {
                    break;
                }
                *V.offset(*I.offset(k + i)) = k + j - 1isize;
                i = i + 1isize;
            }
            if j == 1isize {
                *I.offset(k) = -1isize;
            }
            k = k + j;
        }
    } else {
        x = *V.offset(*I.offset(start + len / 2isize) + h);
        jj = 0isize;
        kk = 0isize;
        i = start;
        'loop2: loop {
            if !(i < start + len) {
                break;
            }
            if *V.offset(*I.offset(i) + h) < x {
                jj = jj + 1isize;
            }
            if *V.offset(*I.offset(i) + h) == x {
                kk = kk + 1isize;
            }
            i = i + 1isize;
        }
        jj = jj + start;
        kk = kk + jj;
        i = start;
        j = 0isize;
        k = 0isize;
        'loop4: loop {
            if !(i < jj) {
                break;
            }
            if *V.offset(*I.offset(i) + h) < x {
                i = i + 1isize;
            } else if *V.offset(*I.offset(i) + h) == x {
                tmp = *I.offset(i);
                *I.offset(i) = *I.offset(jj + j);
                *I.offset(jj + j) = tmp;
                j = j + 1isize;
            } else {
                tmp = *I.offset(i);
                *I.offset(i) = *I.offset(kk + k);
                *I.offset(kk + k) = tmp;
                k = k + 1isize;
            }
        }
        'loop5: loop {
            if !(jj + j < kk) {
                break;
            }
            if *V.offset(*I.offset(jj + j) + h) == x {
                j = j + 1isize;
            } else {
                tmp = *I.offset(jj + j);
                *I.offset(jj + j) = *I.offset(kk + k);
                *I.offset(kk + k) = tmp;
                k = k + 1isize;
            }
        }
        if jj > start {
            split(I, V, start, jj - start, h);
        }
        i = 0isize;
        'loop9: loop {
            if !(i < kk - jj) {
                break;
            }
            *V.offset(*I.offset(jj + i)) = kk - 1isize;
            i = i + 1isize;
        }
        if jj == kk - 1isize {
            *I.offset(jj) = -1isize;
        }
        if start + len > kk {
            split(I, V, kk, start + len - kk, h);
        }
    }
}

unsafe fn qsufsort(mut I: *mut isize, mut V: *mut isize, mut old: *const u8, mut oldsize: isize) {
    let mut buckets: [isize; 256] = [0; 256];
    let mut i: isize;
    let mut h: isize;
    let mut len: isize;
    i = 0isize;
    'loop1: loop {
        if !(i < 256isize) {
            break;
        }
        buckets[i as (usize)] = 0isize;
        i = i + 1isize;
    }
    i = 0isize;
    'loop3: loop {
        if !(i < oldsize) {
            break;
        }
        let _rhs = 1;
        let _lhs = &mut buckets[*old.offset(i) as (usize)];
        *_lhs = *_lhs + _rhs as (isize);
        i = i + 1isize;
    }
    i = 1isize;
    'loop5: loop {
        if !(i < 256isize) {
            break;
        }
        let _rhs = buckets[(i - 1isize) as (usize)];
        let _lhs = &mut buckets[i as (usize)];
        *_lhs = *_lhs + _rhs;
        i = i + 1isize;
    }
    i = 255isize;
    'loop7: loop {
        if !(i > 0isize) {
            break;
        }
        buckets[i as (usize)] = buckets[(i - 1isize) as (usize)];
        i = i - 1isize;
    }
    buckets[0usize] = 0isize;
    i = 0isize;
    'loop9: loop {
        if !(i < oldsize) {
            break;
        }
        *I.offset({
                      let _rhs = 1;
                      let _lhs = &mut buckets[*old.offset(i) as (usize)];
                      *_lhs = *_lhs + _rhs as (isize);
                      *_lhs
                  }) = i;
        i = i + 1isize;
    }
    *I.offset(0isize) = oldsize;
    i = 0isize;
    'loop11: loop {
        if !(i < oldsize) {
            break;
        }
        *V.offset(i) = buckets[*old.offset(i) as (usize)];
        i = i + 1isize;
    }
    *V.offset(oldsize) = 0isize;
    i = 1isize;
    'loop13: loop {
        if !(i < 256isize) {
            break;
        }
        if buckets[i as (usize)] == buckets[(i - 1isize) as (usize)] + 1isize {
            *I.offset(buckets[i as (usize)]) = -1isize;
        }
        i = i + 1isize;
    }
    *I.offset(0isize) = -1isize;
    h = 1isize;
    'loop15: loop {
        if !(*I.offset(0isize) != -(oldsize + 1isize)) {
            break;
        }
        len = 0isize;
        i = 0isize;
        'loop22: loop {
            if !(i < oldsize + 1isize) {
                break;
            }
            if *I.offset(i) < 0isize {
                len = len - *I.offset(i);
                i = i - *I.offset(i);
            } else {
                if len != 0 {
                    *I.offset(i - len) = -len;
                }
                len = *V.offset(*I.offset(i)) + 1isize - i;
                split(I, V, i, len, h);
                i = i + len;
                len = 0isize;
            }
        }
        if len != 0 {
            *I.offset(i - len) = -len;
        }
        h = h + h;
    }
    i = 0isize;
    'loop17: loop {
        if !(i < oldsize + 1isize) {
            break;
        }
        *I.offset(*V.offset(i)) = i;
        i = i + 1isize;
    }
}

unsafe fn matchlen(mut old: *const u8,
                   mut oldsize: isize,
                   mut new: *const u8,
                   mut newsize: isize)
                   -> isize {
    let mut i: isize;
    i = 0isize;
    'loop1: loop {
        if !(i < oldsize && (i < newsize)) {
            break;
        }
        if *old.offset(i) as (i32) != *new.offset(i) as (i32) {
            break;
        }
        i = i + 1isize;
    }
    i
}

unsafe fn search(mut I: *const isize,
                 mut old: *const u8,
                 mut oldsize: isize,
                 mut new: *const u8,
                 mut newsize: isize,
                 mut st: isize,
                 mut en: isize,
                 mut pos: *mut isize)
                 -> isize {
    let mut x: isize;
    let mut y: isize;
    if en - st < 2isize {
        x = matchlen(old.offset(*I.offset(st)),
                     oldsize - *I.offset(st),
                     new,
                     newsize);
        y = matchlen(old.offset(*I.offset(en)),
                     oldsize - *I.offset(en),
                     new,
                     newsize);
        (if x > y {
             *pos = *I.offset(st);
             x
         } else {
             *pos = *I.offset(en);
             y
         })
    } else {
        x = st + (en - st) / 2isize;
        (if memcmp(old.offset(*I.offset(x)) as (*const libc::c_void),
                   new as (*const libc::c_void),
                   if oldsize - *I.offset(x) < newsize {
                       oldsize - *I.offset(x)
                   } else {
                       newsize
                   } as (usize)) < 0i32 {
             search(I, old, oldsize, new, newsize, x, en, pos)
         } else {
             search(I, old, oldsize, new, newsize, st, x, pos)
         })
    }
}

unsafe fn offtout(mut x: isize, mut buf: *mut u8) {
    let mut y: isize;
    if x < 0isize {
        y = -x;
    } else {
        y = x;
    }
    *buf.offset(0isize) = (y % 256isize) as (u8);
    y = y - *buf.offset(0isize) as (isize);
    y = y / 256isize;
    *buf.offset(1isize) = (y % 256isize) as (u8);
    y = y - *buf.offset(1isize) as (isize);
    y = y / 256isize;
    *buf.offset(2isize) = (y % 256isize) as (u8);
    y = y - *buf.offset(2isize) as (isize);
    y = y / 256isize;
    *buf.offset(3isize) = (y % 256isize) as (u8);
    y = y - *buf.offset(3isize) as (isize);
    y = y / 256isize;
    *buf.offset(4isize) = (y % 256isize) as (u8);
    y = y - *buf.offset(4isize) as (isize);
    y = y / 256isize;
    *buf.offset(5isize) = (y % 256isize) as (u8);
    y = y - *buf.offset(5isize) as (isize);
    y = y / 256isize;
    *buf.offset(6isize) = (y % 256isize) as (u8);
    y = y - *buf.offset(6isize) as (isize);
    y = y / 256isize;
    *buf.offset(7isize) = (y % 256isize) as (u8);
    if x < 0isize {
        let _rhs = 0x80i32;
        let _lhs = &mut *buf.offset(7isize);
        *_lhs = (*_lhs as (i32) | _rhs) as (u8);
    }
}

unsafe fn writedata<T>(mut opaque: &mut T,
                       mut buffer: *const libc::c_void,
                       mut length: isize,
                       write: unsafe extern "C" fn(&mut T, *const libc::c_void, i32) -> i32)
                       -> isize {
    let mut _currentBlock;
    let mut result: isize = 0isize;
    'loop1: loop {
        if !(length > 0isize) {
            _currentBlock = 2;
            break;
        }
        let smallsize: i32 = (if length < 0x7fffffffisize {
                                  length
                              } else {
                                  0x7fffffffisize
                              }) as (i32);
        let writeresult: i32 = write(opaque, buffer, smallsize);
        if writeresult == -1i32 {
            _currentBlock = 5;
            break;
        }
        result = result + writeresult as (isize);
        length = length - smallsize as (isize);
        buffer = (buffer as (*mut u8)).offset(smallsize as (isize)) as (*const libc::c_void);
    }
    if _currentBlock == 2 { result } else { -1isize }
}

#[repr(C)]
struct bsdiff_request {
    pub old: *const u8,
    pub oldsize: isize,
    pub new: *const u8,
    pub newsize: isize,
    pub I: *mut isize,
    pub buffer: *mut u8,
}

unsafe fn bsdiff_internal<T>(req: &mut bsdiff_request,
                             opaque: &mut T,
                             write: unsafe extern "C" fn(&mut T, *const libc::c_void, i32) -> i32)
                             -> i32 {
    let mut _currentBlock;
    let mut I: *mut isize;
    let mut V: *mut isize;
    let mut scan: isize;
    let mut pos: isize;
    let mut len: isize;
    let mut lastscan: isize;
    let mut lastpos: isize;
    let mut lastoffset: isize;
    let mut oldscore: isize;
    let mut scsc: isize;
    let mut s: isize;
    let mut Sf: isize;
    let mut lenf: isize;
    let mut Sb: isize;
    let mut lenb: isize;
    let mut overlap: isize;
    let mut Ss: isize;
    let mut lens: isize;
    let mut i: isize;
    let mut buffer: *mut u8;
    let mut buf: [u8; 24] = [0; 24];
    if {
           V = libc::malloc(((req.oldsize + 1isize) as (usize))
                                .wrapping_mul(::std::mem::size_of::<isize>())) as
               (*mut isize);
           V
       } == 0i32 as (*mut libc::c_void) as (*mut isize) {
        -1i32
    } else {
        I = req.I;
        qsufsort(I, V, req.old, req.oldsize);
        libc::free(V as (*mut libc::c_void));
        buffer = req.buffer;
        scan = 0isize;
        len = 0isize;
        pos = 0isize;
        lastscan = 0isize;
        lastpos = 0isize;
        lastoffset = 0isize;
        'loop2: loop {
            if !(scan < req.newsize) {
                _currentBlock = 3;
                break;
            }
            oldscore = 0isize;
            scsc = {
                scan = scan + len;
                scan
            };
            'loop5: loop {
                if !(scan < req.newsize) {
                    break;
                }
                len = search(I as (*const isize),
                             req.old,
                             req.oldsize,
                             req.new.offset(scan),
                             req.newsize - scan,
                             0isize,
                             req.oldsize,
                             &mut pos as (*mut isize));
                'loop7: loop {
                    if !(scsc < scan + len) {
                        break;
                    }
                    if scsc + lastoffset < req.oldsize &&
                       (*req.old.offset(scsc + lastoffset) as (i32) ==
                        *req.new.offset(scsc) as (i32)) {
                        oldscore = oldscore + 1isize;
                    }
                    scsc = scsc + 1isize;
                }
                if len == oldscore && (len != 0isize) || len > oldscore + 8isize {
                    break;
                }
                if scan + lastoffset < req.oldsize &&
                   (*req.old.offset(scan + lastoffset) as (i32) == *req.new.offset(scan) as (i32)) {
                    oldscore = oldscore - 1isize;
                }
                scan = scan + 1isize;
            }
            if !(len != oldscore || scan == req.newsize) {
                continue;
            }
            s = 0isize;
            Sf = 0isize;
            lenf = 0isize;
            i = 0isize;
            'loop14: loop {
                if !(lastscan + i < scan && (lastpos + i < req.oldsize)) {
                    break;
                }
                if *req.old.offset(lastpos + i) as (i32) == *req.new.offset(lastscan + i) as (i32) {
                    s = s + 1isize;
                }
                i = i + 1isize;
                if !(s * 2isize - i > Sf * 2isize - lenf) {
                    continue;
                }
                Sf = s;
                lenf = i;
            }
            lenb = 0isize;
            if scan < req.newsize {
                s = 0isize;
                Sb = 0isize;
                i = 1isize;
                'loop17: loop {
                    if !(scan >= lastscan + i && (pos >= i)) {
                        break;
                    }
                    if *req.old.offset(pos - i) as (i32) == *req.new.offset(scan - i) as (i32) {
                        s = s + 1isize;
                    }
                    if s * 2isize - i > Sb * 2isize - lenb {
                        Sb = s;
                        lenb = i;
                    }
                    i = i + 1isize;
                }
            }
            if lastscan + lenf > scan - lenb {
                overlap = lastscan + lenf - (scan - lenb);
                s = 0isize;
                Ss = 0isize;
                lens = 0isize;
                i = 0isize;
                'loop20: loop {
                    if !(i < overlap) {
                        break;
                    }
                    if *req.new.offset(lastscan + lenf - overlap + i) as (i32) ==
                       *req.old.offset(lastpos + lenf - overlap + i) as (i32) {
                        s = s + 1isize;
                    }
                    if *req.new.offset(scan - lenb + i) as (i32) ==
                       *req.old.offset(pos - lenb + i) as (i32) {
                        s = s - 1isize;
                    }
                    if s > Ss {
                        Ss = s;
                        lens = i + 1isize;
                    }
                    i = i + 1isize;
                }
                lenf = lenf + (lens - overlap);
                lenb = lenb - lens;
            }
            offtout(lenf, buf.as_mut_ptr());
            offtout(scan - lenb - (lastscan + lenf),
                    buf.as_mut_ptr().offset(8isize));
            offtout(pos - lenb - (lastpos + lenf),
                    buf.as_mut_ptr().offset(16isize));
            if writedata(opaque,
                         buf.as_mut_ptr() as (*const libc::c_void),
                         ::std::mem::size_of::<[u8; 24]>() as (isize),
                         write) != 0 {
                _currentBlock = 36;
                break;
            }
            i = 0isize;
            'loop24: loop {
                if !(i < lenf) {
                    break;
                }
                *buffer.offset(i) = (*req.new.offset(lastscan + i) as (i32) -
                                     *req.old.offset(lastpos + i) as (i32)) as
                                    (u8);
                i = i + 1isize;
            }
            if writedata(opaque, buffer as (*const libc::c_void), lenf, write) != 0 {
                _currentBlock = 33;
                break;
            }
            i = 0isize;
            'loop27: loop {
                if !(i < scan - lenb - (lastscan + lenf)) {
                    break;
                }
                *buffer.offset(i) = *req.new.offset(lastscan + lenf + i);
                i = i + 1isize;
            }
            if writedata(opaque,
                         buffer as (*const libc::c_void),
                         scan - lenb - (lastscan + lenf),
                         write) != 0 {
                _currentBlock = 30;
                break;
            }
            lastscan = scan - lenb;
            lastpos = pos - lenb;
            lastoffset = pos - scan;
        }
        (if _currentBlock == 3 {
             0i32
         } else if _currentBlock == 30 {
            -1i32
        } else if _currentBlock == 33 {
            -1i32
        } else {
            -1i32
        })
    }
}