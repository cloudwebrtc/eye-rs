use std::io;
use std::sync::{Arc, RwLock};
use std::thread;
use std::time;

use crate::format::{Format, PixelFormat};
use crate::image::CowImage;
use crate::traits::Stream;

struct Buffer {
    data: Vec<u8>,
    state: BufferState,
}

enum BufferState {
    Empty,
    Filled,
    Error,
}

pub struct PlatformStream<'a> {
    _stream: uvc::ActiveStream<'a, Arc<RwLock<Buffer>>>,
    _handle: Box<uvc::StreamHandle<'a>>,
    buffer: Arc<RwLock<Buffer>>,
    format: uvc::StreamFormat,
}

impl<'a> PlatformStream<'a> {
    pub fn new(handle: uvc::StreamHandle<'a>, format: uvc::StreamFormat) -> Self {
        let mut handle = Box::new(handle);
        let handle_ptr = &mut *handle as *mut uvc::StreamHandle;
        let handle_ref = unsafe { &mut *handle_ptr as &mut uvc::StreamHandle };

        let buffer = Arc::new(RwLock::new(Buffer {
            data: Vec::new(),
            state: BufferState::Empty,
        }));
        let stream = handle_ref
            .start_stream(PlatformStream::on_frame, buffer.clone())
            .unwrap();

        PlatformStream {
            _stream: stream,
            _handle: handle,
            buffer,
            format,
        }
    }

    fn on_frame(frame: &uvc::Frame, data: &mut Arc<RwLock<Buffer>>) {
        if let Ok(buffer) = data.read() {
            if let BufferState::Filled = buffer.state {
                // wait for consumers to read the buffer
                thread::sleep(time::Duration::from_millis(1));
                return;
            }
        }

        let mut buffer = if let Ok(buffer) = data.write() {
            buffer
        } else {
            return;
        };

        match frame.to_rgb() {
            Ok(frame_rgb) => {
                let pixels = frame_rgb.to_bytes();
                buffer.data.resize(pixels.len(), 0u8);
                buffer.data.copy_from_slice(pixels);
                buffer.state = BufferState::Filled;
            }
            Err(_) => {
                buffer.data.clear();
                buffer.state = BufferState::Error;
            }
        }
    }
}

impl<'a, 'b> Stream<'b> for PlatformStream<'a> {
    type Item = io::Result<CowImage<'b>>;

    fn next(&'b mut self) -> Option<Self::Item> {
        loop {
            if let Ok(buffer) = self.buffer.read() {
                match buffer.state {
                    BufferState::Empty => thread::sleep(time::Duration::from_millis(1)),
                    BufferState::Error => return None,
                    BufferState::Filled => break,
                }
            }
        }

        let mut buffer = if let Ok(buffer) = self.buffer.write() {
            buffer
        } else {
            return None;
        };

        let format = Format::new(self.format.width, self.format.height, PixelFormat::Rgb(24));

        buffer.state = BufferState::Empty;
        Some(Ok(CowImage::from_bytes(
            buffer.data.iter().cloned(),
            format,
        )))
    }
}
