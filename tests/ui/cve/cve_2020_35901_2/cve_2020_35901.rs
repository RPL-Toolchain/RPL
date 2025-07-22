use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::{fmt, mem};

extern crate futures;
extern crate pin_project;

use futures::{Stream, ready};
use pin_project::pin_project;

/// Type that provides this trait can be streamed to a peer.
pub trait MessageBody {
    fn size(&self) -> BodySize;
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, Error>>>;
}

#[derive(Debug, PartialEq, Copy, Clone)]
/// Body size hint
pub enum BodySize {
    None,
    Empty,
    Sized(usize),
    Sized64(u64),
    Stream,
}

/// Type represent streaming body.
/// Response does not contain `content-length` header and appropriate transfer encoding is used.
#[pin_project]
pub struct BodyStream<S, E> {
    #[pin]
    stream: S,
    _t: PhantomData<E>,
}

impl<S, E> MessageBody for BodyStream<S, E>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<Error>,
{
    fn size(&self) -> BodySize {
        BodySize::Stream
    }
    /// Attempts to pull out the next value of the underlying [`Stream`].
    ///
    /// Empty values are skipped to prevent [`BodyStream`]'s transmission being
    /// ended on a zero-length chunk, but rather proceed until the underlying
    /// [`Stream`] ends.
    // #[rpl::dump_mir(dump_cfg, dump_ddg)]
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, Error>>> {
        let mut stream = unsafe { Pin::new_unchecked(self) }.project().stream;
        //~^ ERROR: it is unsound to call `Pin::new_unchecked` on a mutable reference that can be freely moved
        loop {
            return Poll::Ready(match ready!(stream.as_mut().poll_next(cx)) {
                Some(Ok(ref bytes)) if bytes.is_empty() => continue,
                opt => opt.map(|res| res.map_err(Into::into)),
            });
        }
    }
}

/// Type represent streaming body. This body implementation should be used
/// if total size of stream is known. Data get sent as is without using transfer encoding.
#[pin_project]
pub struct SizedStream<S> {
    size: u64,
    #[pin]
    // FIXME: when there are multiple candidates of an ADT pattern, the ADT matcher failes to
    // match the second pattern because not all the patterns are checked recursively like
    // the statement candidates or local candidates.
    stream: S,
}

impl<S> SizedStream<S>
where
    S: Stream<Item = Result<Bytes, Error>>,
{
    pub fn new(size: u64, stream: S) -> Self {
        SizedStream { size, stream }
    }
}

impl<S> MessageBody for SizedStream<S>
where
    S: Stream<Item = Result<Bytes, Error>>,
{
    fn size(&self) -> BodySize {
        BodySize::Sized64(self.size)
    }
    /// Attempts to pull out the next value of the underlying [`Stream`].
    ///
    /// Empty values are skipped to prevent [`SizedStream`]'s transmission being
    /// ended on a zero-length chunk, but rather proceed until the underlying
    /// [`Stream`] ends.
    // #[rpl::dump_mir(dump_cfg, dump_ddg)]
    fn poll_next(&mut self, cx: &mut Context<'_>) -> Poll<Option<Result<Bytes, Error>>> {
        // FIXME: this should be an error
        let mut stream = unsafe { Pin::new_unchecked(self) }.project().stream;
        // ERROR: it is unsound to call `Pin::new_unchecked` on a mutable reference that can be freely moved
        loop {
            return Poll::Ready(match ready!(stream.as_mut().poll_next(cx)) {
                Some(Ok(ref bytes)) if bytes.is_empty() => continue,
                val => val,
            });
        }
    }
}

pub struct Bytes;

impl Bytes {
    fn is_empty(&self) -> bool {
        true
    }
}

pub struct Error;
