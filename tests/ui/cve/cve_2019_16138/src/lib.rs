//@revisions: inline regular
//@[inline]compile-flags: -Z inline-mir=true
//@[regular]compile-flags: -Z inline-mir=false
extern crate scoped_threadpool;

pub use hdr::decoder::HDRDecoder;

mod color {
    /// An enumeration over supported color types and their bit depths
    #[derive(Copy, PartialEq, Eq, Debug, Clone, Hash)]
    pub enum ColorType {
        /// Pixel is grayscale
        Gray(u8),

        /// Pixel contains R, G and B channels
        RGB(u8),

        /// Pixel is an index into a color palette
        Palette(u8),

        /// Pixel is grayscale with an alpha channel
        GrayA(u8),

        /// Pixel is RGB with an alpha channel
        RGBA(u8),

        /// Pixel contains B, G and R channels
        BGR(u8),

        /// Pixel is BGR with an alpha channel
        BGRA(u8),
    }
}

mod image {
    use std::error::Error;
    use std::fmt;
    use std::io;

    use crate::color::ColorType;

    /// An enumeration of Image errors
    #[derive(Debug)]
    pub enum ImageError {
        /// The Image is not formatted properly
        FormatError(String),

        /// The Image's dimensions are either too small or too large
        DimensionError,

        /// The Decoder does not support this image format
        UnsupportedError(String),

        /// The Decoder does not support this color type
        UnsupportedColor(ColorType),

        /// Not enough data was provided to the Decoder
        /// to decode the image
        NotEnoughData,

        /// An I/O Error occurred while decoding the image
        IoError(io::Error),

        /// The end of the image has been reached
        ImageEnd,

        /// There is not enough memory to complete the given operation
        InsufficientMemory,
    }

    impl fmt::Display for ImageError {
        fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            match *self {
                ImageError::FormatError(ref e) => write!(fmt, "Format error: {}", e),
                ImageError::DimensionError => write!(
                    fmt,
                    "The Image's dimensions are either too \
                     small or too large"
                ),
                ImageError::UnsupportedError(ref f) => write!(
                    fmt,
                    "The Decoder does not support the \
                     image format `{}`",
                    f
                ),
                ImageError::UnsupportedColor(ref c) => write!(
                    fmt,
                    "The decoder does not support \
                     the color type `{:?}`",
                    c
                ),
                ImageError::NotEnoughData => write!(
                    fmt,
                    "Not enough data was provided to the \
                     Decoder to decode the image"
                ),
                ImageError::IoError(ref e) => e.fmt(fmt),
                ImageError::ImageEnd => write!(fmt, "The end of the image has been reached"),
                ImageError::InsufficientMemory => write!(fmt, "Insufficient memory"),
            }
        }
    }

    impl Error for ImageError {
        fn description(&self) -> &str {
            match *self {
                ImageError::FormatError(..) => "Format error",
                ImageError::DimensionError => "Dimension error",
                ImageError::UnsupportedError(..) => "Unsupported error",
                ImageError::UnsupportedColor(..) => "Unsupported color",
                ImageError::NotEnoughData => "Not enough data",
                ImageError::IoError(..) => "IO error",
                ImageError::ImageEnd => "Image end",
                ImageError::InsufficientMemory => "Insufficient memory",
            }
        }

        fn cause(&self) -> Option<&dyn Error> {
            match *self {
                ImageError::IoError(ref e) => Some(e),
                _ => None,
            }
        }
    }

    impl From<io::Error> for ImageError {
        fn from(err: io::Error) -> ImageError {
            ImageError::IoError(err)
        }
    }

    /// Result of an image decoding/encoding process
    pub type ImageResult<T> = Result<T, ImageError>;
}

mod hdr {
    pub(super) mod decoder {
        use scoped_threadpool::Pool;
        #[cfg(test)]
        use std::borrow::Cow;
        use std::io::{self, BufRead};
        use std::iter::Iterator;

        use crate::image::{ImageError, ImageResult};

        /// An Radiance HDR decoder
        #[derive(Debug)]
        pub struct HDRDecoder<R> {
            r: R,
            width: u32,
            height: u32,
            meta: HDRMetadata,
        }

        /// Refer to [wikipedia](https://en.wikipedia.org/wiki/RGBE_image_format)
        #[repr(C)]
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
        pub struct RGBE8Pixel {
            /// Color components
            pub c: [u8; 3],
            /// Exponent
            pub e: u8,
        }

        impl<R: BufRead> HDRDecoder<R> {
            /// Returns file metadata. Refer to ```HDRMetadata``` for details.
            pub fn metadata(&self) -> HDRMetadata {
                self.meta.clone()
            }

            /// Consumes decoder and returns a vector of RGBE8 pixels
            pub fn read_image_native(mut self) -> ImageResult<Vec<RGBE8Pixel>> {
                // Don't read anything if image is empty
                if self.width == 0 || self.height == 0 {
                    return Ok(vec![]);
                }
                // expression self.width > 0 && self.height > 0 is true from now to the end of this method
                let pixel_count = self.width as usize * self.height as usize;
                let mut ret = vec![Default::default(); pixel_count];
                for chunk in ret.chunks_mut(self.width as usize) {
                    read_scanline(&mut self.r, chunk)?;
                }
                Ok(ret)
            }

            /// Consumes decoder and returns a vector of transformed pixels
            pub fn read_image_transform<T: Send, F: Send + Sync + Fn(RGBE8Pixel) -> T>(
                mut self,
                f: F,
            ) -> ImageResult<Vec<T>> {
                // Don't read anything if image is empty
                if self.width == 0 || self.height == 0 {
                    return Ok(vec![]);
                }
                // expression self.width > 0 && self.height > 0 is true from now to the end of this method
                // scanline buffer
                let uszwidth = self.width as usize;

                let pixel_count = self.width as usize * self.height as usize;
                let mut ret = Vec::with_capacity(pixel_count); //~[regular] uninit_vec
                unsafe {
                    // RGBE8Pixel doesn't implement Drop, so it's Ok to drop half-initialized ret
                    ret.set_len(pixel_count);
                    //~[regular]^ERROR: it violates the precondition of `Vec::set_len` to extend a `Vec`'s length without initializing its content in advance
                } // ret contains uninitialized data, so now it's my responsibility to return fully initialized ret

                {
                    let chunks_iter = ret.chunks_mut(uszwidth);
                    let mut pool = Pool::new(8); //

                    (pool.scoped(|scope| {
                        for chunk in chunks_iter {
                            let mut buf = Vec::<RGBE8Pixel>::with_capacity(uszwidth); //~[regular] uninit_vec
                            unsafe {
                                buf.set_len(uszwidth);
                                //~[regular]^ERROR: it violates the precondition of `Vec::set_len` to extend a `Vec`'s length without initializing its content in advance
                            }
                            (read_scanline(&mut self.r, &mut buf[..]))?;
                            let f = &f;
                            scope.execute(move || {
                                for (dst, &pix) in chunk.iter_mut().zip(buf.iter()) {
                                    *dst = f(pix);
                                }
                            });
                        }
                        Ok(())
                    }) as Result<(), ImageError>)?;
                }

                Ok(ret)
            }
        }

        impl<R: BufRead> IntoIterator for HDRDecoder<R> {
            type Item = ImageResult<RGBE8Pixel>;
            type IntoIter = HDRImageDecoderIterator<R>;

            fn into_iter(self) -> Self::IntoIter {
                HDRImageDecoderIterator {
                    r: self.r,
                    scanline_cnt: self.height as usize,
                    buf: vec![Default::default(); self.width as usize],
                    col: 0,
                    scanline: 0,
                    trouble: true, // make first call to `next()` read scanline
                    error_encountered: false,
                }
            }
        }

        /// Scanline buffered pixel by pixel iterator
        pub struct HDRImageDecoderIterator<R: BufRead> {
            r: R,
            scanline_cnt: usize,
            buf: Vec<RGBE8Pixel>, // scanline buffer
            col: usize,           // current position in scanline
            scanline: usize,      // current scanline
            trouble: bool,        // optimization, true indicates that we need to check something
            error_encountered: bool,
        }

        impl<R: BufRead> HDRImageDecoderIterator<R> {
            // Advances counter to the next pixel
            #[inline]
            fn advance(&mut self) {
                //~^ERROR: it usually isn't necessary to apply #[inline] to private functions
                //~|ERROR: it usually isn't necessary to apply #[inline] to generic functions
                self.col += 1;
                if self.col == self.buf.len() {
                    self.col = 0;
                    self.scanline += 1;
                    self.trouble = true;
                }
            }
        }

        impl<R: BufRead> Iterator for HDRImageDecoderIterator<R> {
            type Item = ImageResult<RGBE8Pixel>;

            fn next(&mut self) -> Option<Self::Item> {
                if !self.trouble {
                    let ret = self.buf[self.col];
                    self.advance();
                    Some(Ok(ret))
                } else {
                    // some condition is pending
                    if self.buf.is_empty() || self.scanline == self.scanline_cnt {
                        // No more pixels
                        return None;
                    } // no else
                    if self.error_encountered {
                        self.advance();
                        // Error was encountered. Keep producing errors.
                        // ImageError can't implement Clone, so just dump some error
                        return Some(Err(ImageError::ImageEnd));
                    } // no else
                    if self.col == 0 {
                        // fill scanline buffer
                        match read_scanline(&mut self.r, &mut self.buf[..]) {
                            Ok(_) => {
                                // no action required
                            }
                            Err(err) => {
                                self.advance();
                                self.error_encountered = true;
                                self.trouble = true;
                                return Some(Err(err));
                            }
                        }
                    } // no else
                    self.trouble = false;
                    let ret = self.buf[0];
                    self.advance();
                    Some(Ok(ret))
                }
            }

            fn size_hint(&self) -> (usize, Option<usize>) {
                let total_cnt = self.buf.len() * self.scanline_cnt;
                let cur_cnt = self.buf.len() * self.scanline + self.col;
                let remaining = total_cnt - cur_cnt;
                (remaining, Some(remaining))
            }
        }

        impl<R: BufRead> ExactSizeIterator for HDRImageDecoderIterator<R> {}

        // Precondition: buf.len() > 0
        fn read_scanline<R: BufRead>(r: &mut R, buf: &mut [RGBE8Pixel]) -> ImageResult<()> {
            assert!(!buf.is_empty());
            let width = buf.len();
            // first 4 bytes in scanline allow to determine compression method
            let fb = read_rgbe(r)?;
            if fb.c[0] == 2 && fb.c[1] == 2 && fb.c[2] < 128 {
                // denormalized pixel value (2,2,<128,_) indicates new per component RLE method
                // decode_component guarantees that offset is within 0 .. width
                // therefore we can skip bounds checking here, but we will not
                decode_component(r, width, |offset, value| buf[offset].c[0] = value)?;
                decode_component(r, width, |offset, value| buf[offset].c[1] = value)?;
                decode_component(r, width, |offset, value| buf[offset].c[2] = value)?;
                decode_component(r, width, |offset, value| buf[offset].e = value)?;
            } else {
                // old RLE method (it was considered old around 1991, should it be here?)
                decode_old_rle(r, fb, buf)?;
            }
            Ok(())
        }

        #[inline(always)]
        fn read_byte<R: BufRead>(r: &mut R) -> io::Result<u8> {
            //~^ERROR: it usually isn't necessary to apply #[inline] to private functions
            //~|ERROR: it usually isn't necessary to apply #[inline] to generic functions
            let mut buf = [0u8];
            r.read_exact(&mut buf[..])?;
            Ok(buf[0])
        }

        // Guarantees that first parameter of set_component will be within pos .. pos+width
        #[inline]
        fn decode_component<R: BufRead, S: FnMut(usize, u8)>(
            //~^ERROR: it usually isn't necessary to apply #[inline] to private functions
            //~|ERROR: it usually isn't necessary to apply #[inline] to generic functions
            r: &mut R,
            width: usize,
            mut set_component: S,
        ) -> ImageResult<()> {
            let mut buf = [0; 128];
            let mut pos = 0;
            while pos < width {
                // increment position by a number of decompressed values
                pos += {
                    let rl = read_byte(r)?;
                    if rl <= 128 {
                        // sanity check
                        if pos + rl as usize > width {
                            return Err(ImageError::FormatError(
                                "Wrong length of decoded scanline".into(),
                            ));
                        }
                        // read values
                        r.read_exact(&mut buf[0..rl as usize])?;
                        for (offset, &value) in buf[0..rl as usize].iter().enumerate() {
                            set_component(pos + offset, value);
                        }
                        rl as usize
                    } else {
                        // ?run
                        let rl = rl - 128;
                        // sanity check
                        if pos + rl as usize > width {
                            return Err(ImageError::FormatError(
                                "Wrong length of decoded scanline".into(),
                            ));
                        }
                        // fill with same value
                        let value = read_byte(r)?;
                        for offset in 0..rl as usize {
                            set_component(pos + offset, value);
                        }
                        rl as usize
                    }
                };
            }
            if pos != width {
                return Err(ImageError::FormatError(
                    "Wrong length of decoded scanline".into(),
                ));
            }
            Ok(())
        }

        // Decodes scanline, places it into buf
        // Precondition: buf.len() > 0
        // fb - first 4 bytes of scanline
        fn decode_old_rle<R: BufRead>(
            r: &mut R,
            fb: RGBE8Pixel,
            buf: &mut [RGBE8Pixel],
        ) -> ImageResult<()> {
            assert!(!buf.is_empty());
            let width = buf.len();
            // convenience function.
            // returns run length if pixel is a run length marker
            #[inline]
            fn rl_marker(pix: RGBE8Pixel) -> Option<usize> {
                //~^ERROR: it usually isn't necessary to apply #[inline] to private functions
                if pix.c == [1, 1, 1] {
                    Some(pix.e as usize)
                } else {
                    None
                }
            }
            // first pixel in scanline should not be run length marker
            // it is error if it is
            if rl_marker(fb).is_some() {
                return Err(ImageError::FormatError(
                    "First pixel of a scanline shouldn't be run length marker".into(),
                ));
            }
            buf[0] = fb; // set first pixel of scanline

            let mut x_off = 1; // current offset from beginning of a scanline
            let mut rl_mult = 1; // current run length multiplier
            let mut prev_pixel = fb;
            while x_off < width {
                let pix = read_rgbe(r)?;
                // it's harder to forget to increase x_off if I write this this way.
                x_off += {
                    if let Some(rl) = rl_marker(pix) {
                        // rl_mult takes care of consecutive RL markers
                        let rl = rl * rl_mult;
                        rl_mult *= 256;
                        if x_off + rl <= width {
                            // do run
                            for b in &mut buf[x_off..x_off + rl] {
                                *b = prev_pixel;
                            }
                        } else {
                            return Err(ImageError::FormatError(
                                "Wrong length of decoded scanline".into(),
                            ));
                        };
                        rl // value to increase x_off by
                    } else {
                        rl_mult = 1; // chain of consecutive RL markers is broken
                        prev_pixel = pix;
                        buf[x_off] = pix;
                        1 // value to increase x_off by
                    }
                };
            }
            if x_off != width {
                return Err(ImageError::FormatError(
                    "Wrong length of decoded scanline".into(),
                ));
            }
            Ok(())
        }

        fn read_rgbe<R: BufRead>(r: &mut R) -> io::Result<RGBE8Pixel> {
            let mut buf = [0u8; 4];
            r.read_exact(&mut buf[..])?;
            Ok(RGBE8Pixel {
                c: [buf[0], buf[1], buf[2]],
                e: buf[3],
            })
        }

        /// Metadata for Radiance HDR image
        #[derive(Debug, Clone)]
        pub struct HDRMetadata {
            /// Width of decoded image. It could be either scanline length,
            /// or scanline count, depending on image orientation.
            pub width: u32,
            /// Height of decoded image. It depends on orientation too.
            pub height: u32,
            /// Orientation matrix. For standard orientation it is ((1,0),(0,1)) - left to right, top to bottom.
            /// First pair tells how resulting pixel coordinates change along a scanline.
            /// Second pair tells how they change from one scanline to the next.
            pub orientation: ((i8, i8), (i8, i8)),
            /// Divide color values by exposure to get to get physical radiance in
            /// watts/steradian/m<sup>2</sup>
            ///
            /// Image may not contain physical data, even if this field is set.
            pub exposure: Option<f32>,
            /// Divide color values by corresponding tuple member (r, g, b) to get to get physical radiance
            /// in watts/steradian/m<sup>2</sup>
            ///
            /// Image may not contain physical data, even if this field is set.
            pub color_correction: Option<(f32, f32, f32)>,
            /// Pixel height divided by pixel width
            pub pixel_aspect_ratio: Option<f32>,
            /// All lines contained in image header are put here. Ordering of lines is preserved.
            /// Lines in the form "key=value" are represented as ("key", "value").
            /// All other lines are ("", "line")
            pub custom_attributes: Vec<(String, String)>,
        }
    }
}
