#![cfg(feature = "std")]

use crate::fat::Realloc;
use crate::vec::AVec;

use std::io::*;



impl<A: Realloc> Write for AVec<u8, A> {
    fn flush(&mut self) -> Result<()> { Ok(()) }
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        match self.try_extend_from_slice(buf) {
            Ok(()) => Ok(buf.len()),
            Err(_err) => Err(Error::from(ErrorKind::OutOfMemory)),
        }
    }
}
