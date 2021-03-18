/*!
reverse cursor over &[u8]
*/

use crate::prelude::*;

use core::mem::MaybeUninit;

pub struct WriteCursor<'a> {
    array: &'a mut [MaybeUninit<u8>],
    pos: usize,
}

impl<'a> WriteCursor<'a> {
    pub fn new(array: &'a mut [MaybeUninit<u8>]) -> WriteCursor {
        let pos = array.len();
        WriteCursor { array, pos }
    }
}

impl io::Write for WriteCursor<'_> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        let mut so_far = 0;
        for c in buf {
            so_far = so_far + 1;
            self.pos = self.pos - 1;
            unsafe { *self.array[self.pos].as_mut_ptr() = *c };
        }
        Ok(so_far)
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

pub struct ReadCursor<'a> {
    array: &'a [u8],
    pos: usize,
}

impl ReadCursor<'_> {
    pub fn from(array: &[u8]) -> ReadCursor {
        ReadCursor {
            array,
            pos: array.len(),
        }
    }
}

impl io::Read for ReadCursor<'_> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let mut so_far = 0;

        for c in buf {
            if self.pos == 0 {
                break;
            }
            self.pos = self.pos - 1;
            *c = self.array[self.pos];
            so_far = so_far + 1;
        }

        Ok(so_far)
    }
}
