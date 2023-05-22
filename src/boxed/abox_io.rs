#![cfg(feature = "std")]

use crate::boxed::ABox;
use crate::fat::*;

use std::fmt::Arguments;
use std::io::{BufRead, Read, Result, IoSlice, IoSliceMut, Seek, SeekFrom, Write};
use std::string::String;
use std::vec::Vec;

// XXX: honestly not 100% sure these are worth keeping, even for parity with Box



#[cfg(unix)] impl<T: std::os::fd::AsFd    + ?Sized, A: Free> std::os::fd::AsFd    for ABox<T, A> { fn as_fd(&self) -> std::os::fd::BorrowedFd<'_> { (**self).as_fd() } }
#[cfg(unix)] impl<T: std::os::fd::AsRawFd + ?Sized, A: Free> std::os::fd::AsRawFd for ABox<T, A> { fn as_raw_fd(&self) -> std::os::fd::RawFd { (**self).as_raw_fd() } }

impl<T: Read + ?Sized, A: Free> Read for ABox<T, A> {
    #[inline] fn read(&mut self, buf: &mut [u8]) -> Result<usize>                       { (**self).read(buf) }
    #[inline] fn read_exact(&mut self, buf: &mut [u8]) -> Result<()>                    { (**self).read_exact(buf) }
    #[inline] fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<usize>             { (**self).read_to_end(buf) }
    #[inline] fn read_to_string(&mut self, buf: &mut String) -> Result<usize>           { (**self).read_to_string(buf) }
    #[inline] fn read_vectored(&mut self, bufs: &mut [IoSliceMut<'_>]) -> Result<usize> { (**self).read_vectored(bufs) }
}

impl<T: Seek + ?Sized, A: Free> Seek for ABox<T, A> {
    #[inline] fn rewind(&mut self) -> Result<()>                                        { (**self).rewind() }
    #[inline] fn seek(&mut self, pos: SeekFrom) -> Result<u64>                          { (**self).seek(pos) }
    #[inline] fn stream_position(&mut self) -> Result<u64>                              { (**self).stream_position() }
}

impl<T: Write + ?Sized, A: Free> Write for ABox<T, A> {
    #[inline] fn flush(&mut self) -> Result<()>                                         { (**self).flush() }
    #[inline] fn write(&mut self, buf: &[u8]) -> Result<usize>                          { (**self).write(buf) }
    #[inline] fn write_all(&mut self, buf: &[u8]) -> Result<()>                         { (**self).write_all(buf) }
    #[inline] fn write_fmt(&mut self, fmt: Arguments<'_>) -> Result<()>                 { (**self).write_fmt(fmt) }
    #[inline] fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> Result<usize>       { (**self).write_vectored(bufs) }
}

impl<T: BufRead + ?Sized, A: Free> BufRead for ABox<T, A> {
    #[inline] fn consume(&mut self, amt: usize)                                         { (**self).consume(amt) }
    #[inline] fn fill_buf(&mut self) -> Result<&[u8]>                                   { (**self).fill_buf() }
    #[inline] fn read_line(&mut self, buf: &mut String) -> Result<usize>                { (**self).read_line(buf) }
    #[inline] fn read_until(&mut self, byte: u8, buf: &mut Vec<u8>) -> Result<usize>    { (**self).read_until(byte, buf) }
}
