use std::cmp::{Eq, PartialEq};
use std::fmt;
use std::hash::Hash;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
/// Pixel format type used to describe image pixels.
///
/// Arbitrary formats can be wrapped in the Custom variant.
/// The other variants have values describing the depth of a whole pixel in bits.
pub enum PixelFormat {
    /// Special type for application defined formats
    Custom(String),

    /// Z buffers
    Depth(u32),
    /// Grayscale
    Gray(u32),

    /// Blue, Green, Red
    Bgr(u32),
    /// Red, Green, Blue
    Rgb(u32),

    /// JPEG compression
    Jpeg,
}

impl PixelFormat {
    /// Returns the number of bits of a whole pixel
    pub fn bits(&self) -> Option<u32> {
        match self {
            // Custom
            PixelFormat::Custom(_) => None,
            // Uncompressed
            PixelFormat::Depth(bits) => Some(*bits),
            PixelFormat::Gray(bits) => Some(*bits),
            PixelFormat::Bgr(bits) => Some(*bits),
            PixelFormat::Rgb(bits) => Some(*bits),
            // Compressed
            PixelFormat::Jpeg => None,
        }
    }
}

impl fmt::Display for PixelFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, Debug)]
/// Image buffer format description
pub struct ImageFormat {
    /// Width in pixels
    pub width: u32,
    /// Height in pixels
    pub height: u32,
    /// PixelFormat
    pub pixfmt: PixelFormat,
    /// Length of a pixel row in bytes
    pub stride: Option<usize>,
}

impl ImageFormat {
    /// Returns an image format representation
    ///
    /// # Arguments
    ///
    /// * `width` - Width in pixels
    /// * `height` - Height in pixels
    /// * `pixfmt` - PixelFormat
    ///
    /// # Example
    ///
    /// ```
    /// use eye_hal::format::{ImageFormat, PixelFormat};
    /// let format = ImageFormat::new(1280, 720, PixelFormat::Rgb(24));
    /// ```
    pub fn new(width: u32, height: u32, pixfmt: PixelFormat) -> Self {
        let stride = if let Some(bits) = pixfmt.bits() {
            Some((width * (bits / 8)) as usize)
        } else {
            None
        };

        ImageFormat {
            width,
            height,
            pixfmt,
            stride,
        }
    }

    /// Builder pattern constructor
    ///
    /// # Arguments
    ///
    /// * `stride` - Length of a pixel row in bytes
    pub fn stride(mut self, stride: usize) -> Self {
        self.stride = Some(stride);
        self
    }
}
