use anyhow::{Context, Result};
use xcap::Monitor;

use super::{CaptureBackend, CaptureFrame};

pub struct XcapCapture {
    monitor: Monitor,
}

// SAFETY: Monitor contains an HMONITOR (raw pointer) which is a system-wide
// handle that is safe to use from any thread. xcap just doesn't mark it Send.
unsafe impl Send for XcapCapture {}

impl XcapCapture {
    pub fn new() -> Result<Self> {
        let monitor = Monitor::from_point(0, 0)
            .context("failed to get monitor at (0,0)")?;
        Ok(Self { monitor })
    }
}

impl CaptureBackend for XcapCapture {
    fn next_frame(&mut self) -> Result<CaptureFrame> {
        let img = self.monitor.capture_image()
            .context("failed to capture screen")?;
        let width = img.width();
        let height = img.height();
        let data = img.into_raw();
        Ok(CaptureFrame { width, height, data })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_xcap_capture_returns_frame() {
        let capture = XcapCapture::new();
        if capture.is_err() {
            eprintln!("Skipping xcap test (no display)");
            return;
        }
        let mut capture = capture.unwrap();
        let frame = capture.next_frame().unwrap();
        assert!(frame.width > 0);
        assert!(frame.height > 0);
        assert_eq!(frame.data.len(), (frame.width * frame.height * 4) as usize);
    }
}
