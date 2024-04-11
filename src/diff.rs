#![allow(non_snake_case)]
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

use std::cmp::Ordering;
use std::io;
use std::io::Write;

/// Diff an "old" and a "new" file, returning a patch.
///
/// The patch can be applied to the "old" file to return the new file, with `patch::patch()`.
/// `old` and `new` correspond to the "old" and "new" file respectively. The patch will be written into `writer`.
pub fn diff<T: Write>(old: &[u8], new: &[u8], writer: &mut T) -> io::Result<()> {
    bsdiff_internal(old, new, writer)
}

#[inline]
fn usz(i: isize) -> usize {
    debug_assert!(i >= 0);
    i as usize
}

struct SplitParams {
    start: usize,
    len: usize,
}

fn split_internal(
    I: &mut [isize],
    V: &mut [isize],
    start: usize,
    len: usize,
    h: usize,
) -> Option<SplitParams> {
    if len < 16 {
        let mut k = start;
        while k < start + len {
            let mut j = 1;
            let mut x = V[usz(I[k] + h as isize)];
            let mut i = 1;
            while k + i < start + len {
                if V[usz(I[k + i] + h as isize)] < x {
                    x = V[usz(I[k + i] + h as isize)];
                    j = 0;
                }
                if V[usz(I[k + i] + h as isize)] == x {
                    I.swap(k + j, k + i);
                    j += 1;
                }
                i += 1;
            }
            for &Ii in &I[k..k + j] {
                V[usz(Ii)] = k as isize + j as isize - 1;
            }
            if j == 1 {
                I[k] = -1;
            }
            k += j;
        }

        None
    } else {
        let x = V[usz(I[start + len / 2] + h as isize)];
        let mut jj = 0;
        let mut kk = 0;
        for &Ii in &I[start..start + len] {
            if V[usz(Ii + h as isize)] < x {
                jj += 1;
            }
            if V[usz(Ii + h as isize)] == x {
                kk += 1;
            }
        }
        let jj = jj + start;
        let kk = kk + jj;
        let mut j = 0;
        let mut k = 0;
        let mut i = start;
        while i < jj {
            match V[usz(I[i] + h as isize)].cmp(&x) {
                Ordering::Less => i += 1,
                Ordering::Equal => {
                    I.swap(i, jj + j);
                    j += 1;
                }
                Ordering::Greater => {
                    I.swap(i, kk + k);
                    k += 1;
                }
            }
        }
        while jj + j < kk {
            if V[usz(I[jj + j] + h as isize)] == x {
                j += 1;
            } else {
                I.swap(jj + j, kk + k);
                k += 1;
            }
        }
        if jj > start {
            split(I, V, start, jj - start, h);
        }
        for &Ii in &I[jj..kk] {
            V[usz(Ii)] = kk as isize - 1;
        }
        if jj == kk - 1 {
            I[jj] = -1;
        }

        if start + len > kk {
            Some(SplitParams {
                start: kk,
                len: start + len - kk,
            })
        } else {
            None
        }
    }
}

fn split(I: &mut [isize], V: &mut [isize], start: usize, len: usize, h: usize) {
    let mut ret = Some(SplitParams { start, len });
    while let Some(params) = ret {
        ret = split_internal(I, V, params.start, params.len, h);
    }
}

fn qsufsort(I: &mut [isize], V: &mut [isize], old: &[u8]) {
    let mut buckets: [isize; 256] = [0; 256];
    for &o in old {
        buckets[o as usize] += 1;
    }
    for i in 1..256 {
        buckets[i] += buckets[i - 1];
    }
    for i in (1..256).rev() {
        buckets[i] = buckets[i - 1];
    }
    buckets[0] = 0;
    for (i, old) in old.iter().copied().enumerate() {
        buckets[old as usize] += 1;
        I[usz(buckets[old as usize])] = i as isize;
    }
    I[0] = old.len() as isize;
    for (i, old) in old.iter().copied().enumerate() {
        V[i] = buckets[old as usize];
    }
    V[old.len()] = 0;
    for i in 1..256 {
        if buckets[i] == buckets[i - 1] + 1 {
            I[usz(buckets[i])] = -1;
        }
    }
    I[0] = -1;
    let mut h = 1;
    while I[0] != -(old.len() as isize + 1) {
        let mut len = 0;
        let mut i = 0;
        while i < old.len() as isize + 1 {
            if I[usz(i)] < 0 {
                len -= I[usz(i)];
                i = i - I[usz(i)];
            } else {
                if len != 0 {
                    I[usz(i - len)] = -len;
                }
                len = V[usz(I[usz(i)])] + 1 - i;
                split(I, V, usz(i), usz(len), h);
                i += len;
                len = 0;
            }
        }
        if len != 0 {
            I[usz(i - len)] = -len;
        }
        h += h;
    }
    for (i, v) in V[0..=old.len()].iter().copied().enumerate() {
        I[usz(v)] = i as isize;
    }
}

fn matchlen(old: &[u8], new: &[u8]) -> usize {
    old.iter().zip(new).take_while(|(a, b)| a == b).count()
}

fn search(I: &[isize], old: &[u8], new: &[u8]) -> (isize, usize) {
    if I.len() < 3 {
        let x = matchlen(&old[usz(I[0])..], new);
        let y = matchlen(&old[usz(I[I.len() - 1])..], new);
        if x > y {
            (I[0], x)
        } else {
            (I[I.len() - 1], y)
        }
    } else {
        let mid = (I.len() - 1) / 2;
        let left = &old[usz(I[mid])..];
        let right = new;
        let len_to_check = left.len().min(right.len());
        if left[..len_to_check] < right[..len_to_check] {
            search(&I[mid..], old, new)
        } else {
            search(&I[..=mid], old, new)
        }
    }
}

#[inline]
fn offtout(x: isize, buf: &mut [u8]) {
    if x >= 0 {
        buf.copy_from_slice(&x.to_le_bytes());
    } else {
        let tmp = (-x) as usize | (1 << 63);
        buf.copy_from_slice(&tmp.to_le_bytes());
    }
}

fn bsdiff_internal(old: &[u8], new: &[u8], writer: &mut dyn Write) -> io::Result<()> {
    let mut I = vec![0; old.len() + 1];
    let mut buffer = Vec::new();
    let mut V = vec![0; old.len() + 1];
    qsufsort(&mut I, &mut V, old);

    let mut scan = 0;
    let mut len = 0usize;
    let mut pos = 0usize;
    let mut lastscan = 0;
    let mut lastpos = 0;
    let mut lastoffset = 0isize;
    while scan < new.len() {
        let mut oldscore = 0;
        scan += len;
        let mut scsc = scan;
        while scan < new.len() {
            let (p, l) = search(&I[..=old.len()], old, &new[scan..]);
            pos = usz(p);
            len = l;
            while scsc < scan + len {
                if scsc as isize + lastoffset < old.len() as _
                    && (old[usz(scsc as isize + lastoffset)] == new[scsc])
                {
                    oldscore += 1;
                }
                scsc += 1;
            }
            if len == oldscore && (len != 0) || len > oldscore + 8 {
                break;
            }
            if scan as isize + lastoffset < old.len() as _
                && (old[usz(scan as isize + lastoffset)] == new[scan])
            {
                oldscore -= 1;
            }
            scan += 1;
        }
        if !(len != oldscore || scan == new.len()) {
            continue;
        }
        let mut s = 0;
        let mut Sf = 0;
        let mut lenf = 0usize;
        let mut i = 0usize;
        while lastscan + i < scan && (lastpos + i < old.len() as _) {
            if old[lastpos + i] == new[lastscan + i] {
                s += 1;
            }
            i += 1;
            if s * 2 - i as isize <= Sf * 2 - lenf as isize {
                continue;
            }
            Sf = s;
            lenf = i;
        }
        let mut lenb = 0;
        if scan < new.len() {
            let mut s = 0isize;
            let mut Sb = 0;
            let mut i = 1;
            while scan >= lastscan + i && (pos >= i) {
                if old[pos - i] == new[scan - i] {
                    s += 1;
                }
                if s * 2 - i as isize > Sb * 2 - lenb as isize {
                    Sb = s;
                    lenb = i;
                }
                i += 1;
            }
        }
        if lastscan + lenf > scan - lenb {
            let overlap = lastscan + lenf - (scan - lenb);
            let mut s = 0;
            let mut Ss = 0;
            let mut lens = 0;
            for i in 0..overlap {
                if new[lastscan + lenf - overlap + i] == old[lastpos + lenf - overlap + i] {
                    s += 1;
                }
                if new[scan - lenb + i] == old[pos - lenb + i] {
                    s -= 1;
                }
                if s > Ss {
                    Ss = s;
                    lens = i + 1;
                }
            }
            lenf = lenf + lens - overlap;
            lenb -= lens;
        }
        let mut buf: [u8; 24] = [0; 24];
        offtout(lenf as _, &mut buf[..8]);
        offtout(
            scan as isize - lenb as isize - (lastscan + lenf) as isize,
            &mut buf[8..16],
        );
        offtout(
            pos as isize - lenb as isize - (lastpos + lenf) as isize,
            &mut buf[16..24],
        );
        writer.write_all(&buf[..24])?;

        buffer.clear();
        buffer.extend(
            new[lastscan..lastscan + lenf]
                .iter()
                .zip(&old[lastpos..lastpos + lenf])
                .map(|(n, o)| n.wrapping_sub(*o)),
        );
        writer.write_all(&buffer)?;

        let write_len = scan - lenb - (lastscan + lenf);
        let write_start = lastscan + lenf;
        writer.write_all(&new[write_start..write_start + write_len])?;

        lastscan = scan - lenb;
        lastpos = pos - lenb;
        lastoffset = pos as isize - scan as isize;
    }

    Ok(())
}
