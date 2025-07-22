extern crate bytes;
extern crate tokio;
extern crate tokio_util;

use std::pin::Pin;
use std::task::{ready, Context, Poll};
use std::{fmt, io};

use bytes::{Buf, BytesMut};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio_util::codec::{Decoder, Encoder};

const LW: usize = 1024;
const HW: usize = 8 * 1024;

struct Flags(u8);

impl Flags {
    const EOF: Flags = Flags(0b0001);
    const READABLE: Flags = Flags(0b0010);

    fn contains(&self, other: Flags) -> bool {
        (self.0 & other.0) == other.0
    }

    fn insert(&mut self, other: Flags) {
        self.0 |= other.0;
    }

    fn remove(&mut self, other: Flags) {
        self.0 &= !other.0;
    }
}

/// A unified `Stream` and `Sink` interface to an underlying I/O object, using
/// the `Encoder` and `Decoder` traits to encode and decode frames.
pub struct Framed<T, U> {
    io: T,
    codec: U,
    flags: Flags,
    read_buf: BytesMut,
    write_buf: BytesMut,
}

impl<T, U> Framed<T, U> {
    /// Try to read underlying I/O stream and decode item.
    // #[rpl::dump_mir(dump_cfg, dump_ddg)]
    pub fn next_item(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<U::Item, U::Error>>>
    where
        T: AsyncRead,
        U: Decoder,
    {
        loop {
            // Repeatedly call `decode` or `decode_eof` as long as it is
            // "readable". Readable is defined as not having returned `None`. If
            // the upstream has returned EOF, and the decoder is no longer
            // readable, it can be assumed that the decoder will never become
            // readable again, at which point the stream is terminated.

            if self.flags.contains(Flags::READABLE) {
                if self.flags.contains(Flags::EOF) {
                    match self.codec.decode_eof(&mut self.read_buf) {
                        Ok(Some(frame)) => return Poll::Ready(Some(Ok(frame))),
                        Ok(None) => return Poll::Ready(None),
                        Err(e) => return Poll::Ready(Some(Err(e))),
                    }
                }

                log::trace!("attempting to decode a frame");

                match self.codec.decode(&mut self.read_buf) {
                    Ok(Some(frame)) => {
                        log::trace!("frame decoded from buffer");
                        return Poll::Ready(Some(Ok(frame)));
                    }
                    Err(e) => return Poll::Ready(Some(Err(e))),
                    _ => (), // Need more data
                }

                self.flags.remove(Flags::READABLE);
            }

            debug_assert!(!self.flags.contains(Flags::EOF));

            // Otherwise, try to read more data and try again. Make sure we've got room
            let remaining = self.read_buf.capacity() - self.read_buf.len();
            if remaining < LW {
                self.read_buf.reserve(HW - remaining)
            }
            let cnt = match unsafe {
                // FIXME: this should be an error
                Pin::new_unchecked(&mut self.io).poll_read_buf(cx, &mut self.read_buf)
                // ERROR: it is unsound to call `Pin::new_unchecked` on a mutable reference that can be freely moved
            } {
                Poll::Pending => return Poll::Pending,
                Poll::Ready(Err(e)) => return Poll::Ready(Some(Err(e.into()))),
                Poll::Ready(Ok(cnt)) => cnt,
            };

            if cnt == 0 {
                self.flags.insert(Flags::EOF);
            }
            self.flags.insert(Flags::READABLE);
        }
    }

    /// Flush write buffer to underlying I/O stream.
    pub fn flush(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), U::Error>>
    where
        T: AsyncWrite,
        U: Encoder,
    {
        log::trace!("flushing framed transport");

        while !self.write_buf.is_empty() {
            log::trace!("writing; remaining={}", self.write_buf.len());

            let n = ready!(unsafe {
                Pin::new_unchecked(&mut self.io).poll_write(cx, &self.write_buf)
                //~^ ERROR: it is unsound to call `Pin::new_unchecked` on a mutable reference that can be freely moved
            })?;

            if n == 0 {
                return Poll::Ready(Err(io::Error::new(
                    io::ErrorKind::WriteZero,
                    "failed to write frame to transport",
                )
                .into()));
            }

            // remove written data
            self.write_buf.advance(n);
        }

        // Try flushing the underlying IO
        ready!(unsafe { Pin::new_unchecked(&mut self.io).poll_flush(cx) })?;
        //~^ ERROR: it is unsound to call `Pin::new_unchecked` on a mutable reference that can be freely moved

        log::trace!("framed transport flushed");
        Poll::Ready(Ok(()))
    }

    /// Flush write buffer and shutdown underlying I/O stream.
    // #[rpl::dump_mir(dump_cfg, dump_ddg)]
    pub fn close(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), U::Error>>
    where
        T: AsyncWrite,
        U: Encoder,
    {
        unsafe {
            ready!(Pin::new_unchecked(&mut self.io).poll_flush(cx))?;
            //~^ ERROR: it is unsound to call `Pin::new_unchecked` on a mutable reference that can be freely moved
            ready!(Pin::new_unchecked(&mut self.io).poll_shutdown(cx))?;
            //~^ ERROR: it is unsound to call `Pin::new_unchecked` on a mutable reference that can be freely moved
        }
        Poll::Ready(Ok(()))
    }
}
