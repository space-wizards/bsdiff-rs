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

use std::io;
use std::io::Read;

/// Apply a patch to a set of bytes.
/// `old` is the old file, `patch` is a `Read` with the patch, `new` is the buffer that will be written into.
/// `new` must be large enough to store the resulting file. You should probably store the size of the patched file somewhere when diffing the files.
pub fn bspatch<T>(old: &[u8], patch: &mut T, new: &mut [u8]) -> io::Result<()>
    where T: Read
{
    // I am more than well aware about the giant amounts of as operators for int casting.
    let mut buf = [0u8; 8];
    let mut oldpos = 0i64;
    let mut newpos = 0i64;
    let mut ctrl = [0i64; 3];

    while newpos < new.len() as i64 {
        // Read control data
        for i in 0..3 {
            patch.read_exact(&mut buf)?;
            ctrl[i] = offtin(&buf);
        }

        // Sanity-check
        if newpos as i64 + ctrl[0] > new.len() as i64 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Output file too short."));
        }

        // Read diff string
        patch.read_exact(&mut new[newpos as usize ..(newpos+ctrl[0]) as usize])?;

        // Add old data to diff string
        for i in 0..ctrl[0] {
            if oldpos + 1 > 0 && oldpos + i < old.len() as i64 {
                new[(newpos + i) as usize] += old[(oldpos + i) as usize];
            }
        }

        // Adjust pointers
        newpos += ctrl[0];
        oldpos += ctrl[0];

        // Sanity-check
        if newpos + ctrl[1] > new.len() as i64 {
            return Err(io::Error::new(io::ErrorKind::UnexpectedEof, "Output file too short."));
        }

        // Read extra string
        patch.read_exact(&mut new[newpos as usize ..(newpos+ctrl[1]) as usize])?;

        // Adjust pointers
        newpos += ctrl[1];
        oldpos += ctrl[2];
    }

    Ok(())
}

fn offtin(buf: &[u8]) -> i64 {
    let mut y;
    y = (buf[7] as (i32) & 0x7Fi32) as (i64);
    y = y * 256;
    y = y + buf[6] as (i64);
    y = y * 256;
    y = y + buf[5] as (i64);
    y = y * 256;
    y = y + buf[4] as (i64);
    y = y * 256;
    y = y + buf[3] as (i64);
    y = y * 256;
    y = y + buf[2] as (i64);
    y = y * 256;
    y = y + buf[1] as (i64);
    y = y * 256;
    y = y + buf[0] as (i64);
    if buf[7] as (i32) & 0x80i32 != 0 {
        y = -y;
    }
    y
}

/*
fn offtin(buf: &[u8]) -> i64 {
    let mut y;
    y = (buf[7] & 0x7F) as (i64);
    for x in (0..7).rev() {
        y *= 256;
        y += buf[x] as (i64);
    }

    println!("{:X}", buf[7]);

    if buf[7] as i32 & 0x80 != 0 {
        -y
    } else {
        y
    }
}
*/