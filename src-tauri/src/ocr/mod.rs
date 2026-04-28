pub mod preprocess;
pub mod segment;
pub mod template;
pub mod tesseract;

use anyhow::Result;
use crate::state::RegionKind;

pub trait OcrPipeline: Send {
    fn read_number(&self, image: &[u8], width: u32, height: u32, kind: RegionKind) -> Result<Option<u32>>;
}
