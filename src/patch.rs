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
/// `new` must be large enough to store the resulting file. You should probably store the size of the patched file somewhere when diffing the files.
pub fn patch<T>(old: &[u8], patch: &mut T, mut new: &mut [u8]) -> io::Result<()>
    where T: Read
{
    let mut oldpos = 0;

    while !new.is_empty() {
        // Read control data
        let mut buf = [0u8; 24];
        patch.read_exact(&mut buf)?;
        // only seek can be negative
        let mixlen = u64::from_le_bytes(buf[0..8].try_into().unwrap()) as usize;
        let copylen = u64::from_le_bytes(buf[8..16].try_into().unwrap()) as usize;
        let seeklen = offtin(buf[16..24].try_into().unwrap());

        // oldpos needs to be checked before the for loop to optimize it better
        if mixlen > new.len() || oldpos > old.len() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        let (mix, rest) = new.split_at_mut(mixlen);
        // Read diff string
        patch.read_exact(mix)?;

        for (n, o) in mix.iter_mut().zip(&old[oldpos..]) {
            *n = n.wrapping_add(*o);
        }

        // Adjust pointers
        new = rest;
        oldpos += mixlen;

        if copylen > new.len() {
            return Err(io::ErrorKind::UnexpectedEof.into());
        }
        let (copy, rest) = new.split_at_mut(copylen);
        patch.read_exact(copy)?;

        // Adjust pointers
        new = rest;
        oldpos = (oldpos as i64 + seeklen) as usize;
    }

    Ok(())
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
