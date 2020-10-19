use std::io;

use ffimage::packed::dynamic::{ImageBuffer, ImageView, MemoryView, StorageType};

use crate::format::{Format, PixelFormat};
use crate::hal::common::convert::Converter;
use crate::hal::traits::Stream;

/// A transparent wrapper for native platform streams.
pub struct TransparentStream<'a> {
    stream: Box<dyn 'a + for<'b> Stream<'b, Item = ImageView<'b>>>,
    format: Format,
    mapping: Option<(PixelFormat, PixelFormat)>,
    convert_buffer: ImageBuffer,
}

impl<'a> TransparentStream<'a> {
    pub fn new(
        stream: Box<dyn 'a + for<'b> Stream<'b, Item = ImageView<'b>>>,
        format: Format,
    ) -> Self {
        TransparentStream {
            stream,
            format,
            mapping: None,
            convert_buffer: ImageBuffer::empty(StorageType::U8),
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
    type Item = ImageView<'b>;

    fn next(&'b mut self) -> io::Result<Self::Item> {
        let mut view = self.stream.next()?;

        // emulate format by converting the buffer if necessary
        if let Some(map) = self.mapping {
            match view.raw() {
                MemoryView::U8(_) => {
                    self.convert_buffer = ImageBuffer::empty(StorageType::U8);
                }
                MemoryView::U16(_) => {
                    self.convert_buffer = ImageBuffer::empty(StorageType::U16);
                }
            }

            Converter::convert(&view, self.format.pixfmt, &mut self.convert_buffer, map.1)?;
            view = self.convert_buffer.as_view()
        }

        Ok(view)
    }
}
