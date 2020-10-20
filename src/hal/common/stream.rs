use std::io;

use crate::format::{Format, PixelFormat};
use crate::hal::common::convert::Converter;
use crate::image::CowImage;
use crate::traits::{ImageStream, Stream};

/// A transparent wrapper for native platform streams.
pub struct TransparentStream<'a> {
    stream: Box<ImageStream<'a>>,
    format: Format,
    mapping: Option<(PixelFormat, PixelFormat)>,
}

impl<'a> TransparentStream<'a> {
    pub fn new(stream: Box<ImageStream<'a>>, format: Format) -> Self {
        TransparentStream {
            stream,
            format,
            mapping: None,
        }
    }

    /// Tell the stream to emulate a format by requesting the compatible source format from the
    /// native device.
    ///
    /// # Arguments
    ///
    /// * `src` - Source format to be set on the device
    /// * `dst` - Target format to emulate
    ///
    pub fn map(&mut self, src: PixelFormat, dst: PixelFormat) {
        self.mapping = Some((src, dst))
    }
}

impl<'a, 'b> Stream<'b> for TransparentStream<'a> {
    type Item = CowImage<'b>;

    fn next(&'b mut self) -> io::Result<Self::Item> {
        let image = self.stream.next()?;

        // emulate format by converting the buffer if necessary
        if let Some(map) = self.mapping {
            let mut buf = image.clone();

            Converter::convert(&image.as_view(), self.format.pixfmt, buf.to_mut(), map.1)?;
            Ok(CowImage::from(buf))
        } else {
            Ok(image)
        }
    }
}
