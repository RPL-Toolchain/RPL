//@rustc-env: RPL_PATS=docs/patterns-safedrop
//@compile-flags: -Z mir-opt-level=0 -Z inline-mir=true
#![allow(
    dead_code,
    unused_imports,
    unused_variables,
    unused_mut,
    deprecated,
    invalid_value,
    non_camel_case_types,
    unsafe_op_in_unsafe_fn
)]
use std::io::{self, Read};
use std::mem;

#[derive(Clone)]
struct Header;
impl Header {
    fn read_from<R: Read>(_reader: &mut R) -> io::Result<Self> {
        Err(io::Error::new(io::ErrorKind::UnexpectedEof, "eof"))
    }
}

struct Decoder<R> {
    inner: R,
}
impl<R> Decoder<R> {
    fn with_header(reader: R, _header: Header) -> Self {
        Self { inner: reader }
    }
    fn into_inner(self) -> R {
        self.inner
    }
}
impl<R: Read> Read for Decoder<R> {
    fn read(&mut self, _buf: &mut [u8]) -> io::Result<usize> {
        Ok(0)
    }
}

struct MultiDecoder<R> {
    decoder: Result<Decoder<R>, R>,
    header: Header,
}

impl<R: Read> Read for MultiDecoder<R> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        let read_size = match self.decoder {
            Err(_) => return Ok(0),
            Ok(ref mut decoder) => decoder.read(buf)?,
        };
        if read_size == 0 {
            let mut reader = mem::replace(
                &mut self.decoder,
                Err(unsafe { mem::MaybeUninit::uninit().assume_init() }),
            )
            .ok()
            .take()
            .expect("Never fails")
            .into_inner();
            //~^ ERROR: this unsafe operation may free storage that is already dead
            //~| ERROR: this unsafe operation may free storage that is already dead
            match Header::read_from(&mut reader) {
                Err(e) => {
                    mem::forget(mem::replace(&mut self.decoder, Err(reader)));
                    if e.kind() == io::ErrorKind::UnexpectedEof {
                        Ok(0)
                    } else {
                        Err(e)
                    }
                }
                Ok(header) => {
                    self.header = header.clone();
                    mem::forget(mem::replace(
                        &mut self.decoder,
                        Ok(Decoder::with_header(reader, header)),
                    ));
                    self.read(buf)
                }
            }
        } else {
        //~^ ERROR: this unsafe operation may free storage that is already dead
            Ok(read_size)
        }
    }
}

fn libflate_multi_decoder_read() {
    let reader = io::Cursor::new(Vec::<u8>::new());
    let decoder = Decoder::with_header(reader, Header);
    let mut multi = MultiDecoder {
        decoder: Ok(decoder),
        header: Header,
    };
    let mut out = [0_u8; 1];
    let _ = multi.read(&mut out);
}

fn main() {
    libflate_multi_decoder_read();
}
