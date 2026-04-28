pub mod fallback;
#[cfg(target_os = "windows")]
pub mod windows;

use anyhow::Result;

pub struct CaptureFrame {
    pub width: u32,
    pub height: u32,
    pub data: Vec<u8>,
}

pub trait CaptureBackend: Send {
    fn next_frame(&mut self) -> Result<CaptureFrame>;
}
