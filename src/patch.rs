/*-
 * Copyright 2003-2005 Colin Percival
 * Copyright 2012 Matthew Endsley
 * Modified 2017 Pieter-Jan Briers
 * Modified 2021 Kornel Lesinski
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

use std::convert::TryInto;
use std::io;
use std::io::Read;

/// Apply a patch to an "old" file, returning the "new" file.
/// 
/// `old` is the old file, `patch` will be read from with the patch,`new` is the buffer that will be written into.
pub fn patch<T: Read>(old: &[u8], patch: &mut T, new: &mut Vec<u8>) -> io::Result<()> {
    let mut oldpos: usize = 0;
    loop {
        // Read control data
        let mut buf = [0; 24];
        if read_or_eof(patch, &mut buf)? {
            return Ok(())
        }

        // only seek can be negative
        let mix_len = u64::from_le_bytes(buf[0..8].try_into().unwrap()) as usize;
        let copy_len = u64::from_le_bytes(buf[8..16].try_into().unwrap()) as usize;
        let seek_len = offtin(buf[16..24].try_into().unwrap());

        // Read diff string and literal data at once
        let to_read = copy_len
            .checked_add(mix_len)
            .ok_or(io::Error::from(io::ErrorKind::InvalidData))?;
        let mix_start = new.len();
        let has_read = patch.take(to_read as u64).read_to_end(new)?;

        // oldpos needs to be checked before the for loop to optimize it better
        if has_read != to_read {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }

        let mix_slice = new
            .get_mut(
                mix_start
                    ..mix_start
                        .checked_add(mix_len)
                        .ok_or(io::Error::from(io::ErrorKind::InvalidData))?,
            )
            .ok_or(io::ErrorKind::UnexpectedEof)?;
        let old_slice = old
            .get(
                oldpos
                    ..oldpos
                        .checked_add(mix_len)
                        .ok_or(io::Error::from(io::ErrorKind::InvalidData))?,
            )
            .ok_or(io::ErrorKind::UnexpectedEof)?;
        for (n, o) in mix_slice.iter_mut().zip(old_slice) {
            *n = n.wrapping_add(*o);
        }

        // Adjust pointers
        oldpos += mix_len;
        oldpos = (oldpos as i64)
            .checked_add(seek_len)
            .ok_or(io::Error::from(io::ErrorKind::InvalidData))? as usize;
    }
}

/// It allows EOF only before the first byte.
fn read_or_eof<T: Read>(reader: &mut T, buf: &mut [u8; 24]) -> io::Result<bool> {
    let mut tmp = &mut buf[..];
    loop {
        match reader.read(tmp) {
            Ok(0) => {
                return if tmp.len() == 24 {
                    Ok(true)
                } else {
                    Err(io::ErrorKind::UnexpectedEof.into())
                }
            },
            Ok(n) => {
                if n >= tmp.len() {
                    return Ok(false);
                }
                tmp = &mut tmp[n..];
            }
            Err(ref e) if e.kind() == io::ErrorKind::Interrupted => {}
            Err(e) => return Err(e),
        }
    }
}

/// Reads sign-magnitude i64 little-endian
#[inline]
fn offtin(buf: [u8; 8]) -> i64 {
    let y = i64::from_le_bytes(buf);
    if 0 == y & (1<<63) {
        y
    } else {
        -(y & !(1<<63))
    }
}
