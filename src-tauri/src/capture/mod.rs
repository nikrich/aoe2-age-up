pub mod fallback;
#[cfg(target_os = "windows")]
pub mod windows;
pub mod loop_task;

use anyhow::Result;

/// A captured frame in RGBA format (4 bytes per pixel).
pub struct CaptureFrame {
    pub width: u32,
    pub height: u32,
    /// Raw pixel data in RGBA format, length = width * height * 4.
    pub data: Vec<u8>,
}

pub trait CaptureBackend: Send {
    fn next_frame(&mut self) -> Result<CaptureFrame>;
}
