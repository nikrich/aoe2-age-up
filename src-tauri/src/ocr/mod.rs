pub mod preprocess;
pub mod segment;
pub mod template;
pub mod fixture;
pub mod tesseract;

use anyhow::Result;
use image::RgbaImage;
use std::collections::HashMap;

use crate::state::RegionKind;
use preprocess::{to_grayscale, threshold};
use segment::segment_characters;
use template::{match_character, chars_to_number, chars_to_time_seconds, chars_to_population};

pub trait OcrPipeline: Send {
    fn read_number(&self, image: &[u8], width: u32, height: u32, kind: RegionKind) -> Result<Option<u32>>;
    fn read_population(&self, image: &[u8], width: u32, height: u32) -> Result<Option<(u32, u32)>>;
    fn read_time(&self, image: &[u8], width: u32, height: u32) -> Result<Option<u32>>;
}

pub struct TemplatePipeline {
    templates: HashMap<char, image::GrayImage>,
    threshold_value: u8,
    confidence: f64,
}

impl TemplatePipeline {
    pub fn new(templates: HashMap<char, image::GrayImage>) -> Self {
        Self {
            templates,
            threshold_value: 160,
            confidence: 0.5,
        }
    }

    fn recognize_chars(&self, image: &[u8], width: u32, height: u32) -> Vec<Option<char>> {
        let rgba = RgbaImage::from_raw(width, height, image.to_vec())
            .unwrap_or_else(|| RgbaImage::new(0, 0));
        if rgba.width() == 0 {
            return Vec::new();
        }
        let gray = to_grayscale(&rgba);
        let binary = threshold(&gray, self.threshold_value);
        let boxes = segment_characters(&binary, 2);
        boxes.iter()
            .map(|b| match_character(&binary, b, &self.templates, self.confidence))
            .collect()
    }
}

impl OcrPipeline for TemplatePipeline {
    fn read_number(&self, image: &[u8], width: u32, height: u32, _kind: RegionKind) -> Result<Option<u32>> {
        let chars = self.recognize_chars(image, width, height);
        Ok(chars_to_number(&chars))
    }

    fn read_population(&self, image: &[u8], width: u32, height: u32) -> Result<Option<(u32, u32)>> {
        let chars = self.recognize_chars(image, width, height);
        Ok(chars_to_population(&chars))
    }

    fn read_time(&self, image: &[u8], width: u32, height: u32) -> Result<Option<u32>> {
        let chars = self.recognize_chars(image, width, height);
        Ok(chars_to_time_seconds(&chars))
    }
}
