/*
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/.
 */

use std::error::Error;
use std::io::{self, Write};

use serde::{Deserialize, Serialize};

mod tests;

pub struct SerdeQueue {
    // If start == end, then there is no data in v
    v: Vec<u8>,
    start: usize,
    end: usize,
    num_elements: usize,
}
// Used to write a single element
struct Writer<'a> {
    q: &'a mut SerdeQueue,
    p: usize,
}
impl<'a> Write for Writer<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.p < self.q.start {
            // We need to reserve one byte in v to avoid start == end
            if buf.len() < self.q.start - self.p {
                let p = self.p + buf.len();
                self.q.v[self.p..p].copy_from_slice(buf);
                self.p = p;
                return Ok(buf.len());
            }
            let needed = self.q.v.len() - self.q.start + self.p + buf.len() + 1;
            let cap = self.q.v.capacity();
            loop {
                // cap *= 1.5
                let cap = cap + cap >> 1;
                if cap >= needed {
                    break;
                }
            }
            let mut v = Vec::with_capacity(cap);
            v.extend_from_slice(&self.q.v[self.q.start..]);
            v.extend_from_slice(&self.q.v[0..self.p]);
            if self.q.end < self.q.start {
                self.q.end = self.q.v.len() - self.q.start + self.q.end;
            } else {
                assert_eq!(self.q.end, self.q.v.len());
                self.q.end = self.q.end - self.q.start;
            }
            self.q.v = v;
            self.q.start = 0;
            self.q.v.extend_from_slice(buf);
            self.p = self.q.v.len();
        } else {
            assert!(self.p == self.q.v.len());
            // No need to reserve one byte in "v" because start won't == end
            if buf.len() <= self.q.v.capacity() - self.p {
                self.q.v.extend_from_slice(buf);
                self.p = self.q.v.len();
                return Ok(buf.len());
            }
            // We need to make the content of an element physically continuous.
            // Therefore, if the next coming element has a size larger than the
            // trailing available space of "v", we try to rewind to the start
            // of "v" and write the element there.
            assert!(self.q.end <= self.p);
            let written = self.p - self.q.end;
            // Reserve one byte to avoid p == start
            if written + buf.len() < self.q.start {
                let (a, b) = self.q.v.split_at_mut(self.q.end);
                a[0..written].copy_from_slice(b);
                self.p = written + buf.len();
                self.q.v[written..self.p].copy_from_slice(buf);
                self.q.v.truncate(self.q.end);
                return Ok(buf.len());
            }
            // The start of "v" has no enough space.
            // We simply append to the end of "v" and make rust automatically
            // extends the capacity. In this way, it's possible that no memory
            // move is required.
            self.q.v.extend_from_slice(buf);
            self.p = self.q.v.len();
        }
        Ok(buf.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
impl SerdeQueue {
    pub fn new() -> SerdeQueue {
        SerdeQueue {
            v: Vec::new(),
            start: 0,
            end: 0,
            num_elements: 0,
        }
    }
    pub fn len(&self) -> usize {
        self.num_elements
    }
    pub fn push<'a, 'b, T>(&'a mut self, v: &'b T) -> Result<(), Box<dyn Error>>
    where
        T: Serialize,
    {
        let p = self.end;
        let writer = Writer { q: self, p };
        let writer = postcard::to_io(v, writer)?;
        self.end = writer.p;
        self.num_elements += 1;
        Ok(())
    }
    pub fn pop<'a, T>(&'a mut self) -> Result<Option<T>, Box<dyn Error>>
    where
        T: Deserialize<'a>,
    {
        if self.num_elements == 0 {
            return Ok(None);
        }
        self.num_elements -= 1;
        let end;
        if self.start == self.v.len() {
            if self.start == self.end {
                // Possible when the serialized length of elements are zero
                self.end = 0;
            }
            self.start = 0;
            self.v.truncate(self.end);
            end = self.end;
        } else {
            end = self.v.len();
        }
        let (v, remain) = postcard::take_from_bytes(&self.v[self.start..end])?;
        self.start = end - remain.len();
        Ok(Some(v))
    }
}
