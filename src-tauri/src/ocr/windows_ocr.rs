use anyhow::{Context, Result};
use image::RgbaImage;
use std::path::PathBuf;
use std::process::Command;
use tracing::{debug, warn};

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use crate::state::RegionKind;
use super::OcrPipeline;

#[cfg(target_os = "windows")]
const CREATE_NO_WINDOW: u32 = 0x08000000;

/// OCR backend using Tesseract CLI (bundled with the app).
pub struct TesseractPipeline {
    tesseract_path: PathBuf,
    tessdata_path: PathBuf,
    /// Directory containing tesseract's DLL dependencies (added to PATH on spawn).
    deps_dir: PathBuf,
}

impl TesseractPipeline {
    pub fn new(tesseract_path: PathBuf, tessdata_path: PathBuf, deps_dir: PathBuf) -> Result<Self> {
        // Strip \\?\ prefix that Tauri's resource_dir() adds — Tesseract can't handle it
        let strip_prefix = |p: PathBuf| -> PathBuf {
            let s = p.to_string_lossy();
            if let Some(stripped) = s.strip_prefix(r"\\?\") {
                PathBuf::from(stripped)
            } else {
                p
            }
        };
        let tesseract_path = strip_prefix(tesseract_path);
        let tessdata_path = strip_prefix(tessdata_path);
        let deps_dir = strip_prefix(deps_dir);

        debug!("Tesseract binary: {:?}", tesseract_path);
        debug!("Tessdata dir: {:?}", tessdata_path);
        debug!("Deps dir: {:?}", deps_dir);

        // Build PATH that includes the deps dir so tesseract can find its DLLs
        let path_env = build_path_with_deps(&deps_dir);

        // Verify tesseract is accessible
        let mut cmd = Command::new(&tesseract_path);
        cmd.arg("--version")
            .env("TESSDATA_PREFIX", &tessdata_path)
            .env("PATH", &path_env);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let output = cmd.output()
            .context("Failed to find tesseract. Is it installed?")?;

        let version = String::from_utf8_lossy(&output.stdout);
        debug!("Tesseract: {}", version.lines().next().unwrap_or("unknown"));

        Ok(Self { tesseract_path, tessdata_path, deps_dir })
    }

    fn recognize_text(&self, image: &[u8], width: u32, height: u32) -> Result<String> {
        let rgba = RgbaImage::from_raw(width, height, image.to_vec())
            .context("Invalid RGBA image data")?;

        // Preprocess: grayscale → threshold → invert to get dark text on white background
        let gray = super::preprocess::to_grayscale(&rgba);
        let binary = super::preprocess::threshold(&gray, 160);

        // Upscale small images for better accuracy
        let scale = if width < 200 { (200 / width).max(4) } else { 1 };
        let img = if scale > 1 {
            let new_w = width * scale;
            let new_h = height * scale;
            let scaled = image::imageops::resize(&binary, new_w, new_h, image::imageops::FilterType::Lanczos3);
            image::DynamicImage::ImageLuma8(scaled)
        } else {
            image::DynamicImage::ImageLuma8(binary)
        };

        // Write PNG to temp file (tesseract reads from file)
        let tmp = std::env::temp_dir().join("aoe_ocr_input.png");
        img.save(&tmp).context("Failed to save temp image")?;

        let path_env = build_path_with_deps(&self.deps_dir);

        let mut cmd = Command::new(&self.tesseract_path);
        cmd.arg(tmp.to_str().unwrap())
            .arg("stdout")
            .arg("--psm").arg("7") // Single line of text
            .arg("-c").arg("tessedit_char_whitelist=0123456789:/")
            .arg("-l").arg("eng")
            .env("TESSDATA_PREFIX", &self.tessdata_path)
            .env("PATH", &path_env);
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);
        let output = cmd.output()
            .context("Failed to run tesseract")?;

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            debug!("Tesseract stderr: {}", stderr.trim());
        }

        if !text.is_empty() {
            debug!("Tesseract recognized: \"{}\" ({}x{}, scale {}x)", text, width, height, scale);
        }

        Ok(text)
    }

    fn extract_number(&self, text: &str) -> Option<u32> {
        let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
        if digits.is_empty() { return None; }
        digits.parse().ok()
    }

    fn extract_population(&self, text: &str) -> Option<(u32, u32)> {
        let text = text.replace(' ', "");
        let parts: Vec<&str> = text.split('/').collect();
        if parts.len() == 2 {
            let current: u32 = parts[0].chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok()?;
            let max: u32 = parts[1].chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok()?;
            Some((current, max))
        } else {
            None
        }
    }

    fn extract_time(&self, text: &str) -> Option<u32> {
        let text = text.trim().trim_end_matches(':');
        let parts: Vec<&str> = text.split(':').collect();
        match parts.len() {
            3 => {
                let h: u32 = parts[0].trim().parse().ok()?;
                let m: u32 = parts[1].trim().parse().ok()?;
                let s: u32 = parts[2].trim().parse().ok()?;
                Some(h * 3600 + m * 60 + s)
            }
            2 => {
                let m: u32 = parts[0].trim().parse().ok()?;
                let s: u32 = parts[1].trim().parse().ok()?;
                Some(m * 60 + s)
            }
            _ => None,
        }
    }
}

/// Prepend the deps directory to the system PATH so tesseract can find its DLLs.
fn build_path_with_deps(deps_dir: &std::path::Path) -> String {
    let system_path = std::env::var("PATH").unwrap_or_default();
    format!("{};{}", deps_dir.display(), system_path)
}

impl OcrPipeline for TesseractPipeline {
    fn read_number(&self, image: &[u8], width: u32, height: u32, kind: RegionKind) -> Result<Option<u32>> {
        match self.recognize_text(image, width, height) {
            Ok(text) => {
                let num = self.extract_number(&text);
                debug!("{:?} -> {:?} (raw: \"{}\")", kind, num, text);
                Ok(num)
            }
            Err(e) => {
                warn!("OCR failed for {:?}: {}", kind, e);
                Ok(None)
            }
        }
    }

    fn read_population(&self, image: &[u8], width: u32, height: u32) -> Result<Option<(u32, u32)>> {
        match self.recognize_text(image, width, height) {
            Ok(text) => {
                let pop = self.extract_population(&text);
                debug!("Population -> {:?} (raw: \"{}\")", pop, text);
                Ok(pop)
            }
            Err(e) => {
                warn!("OCR failed for Population: {}", e);
                Ok(None)
            }
        }
    }

    fn read_time(&self, image: &[u8], width: u32, height: u32) -> Result<Option<u32>> {
        match self.recognize_text(image, width, height) {
            Ok(text) => {
                let time = self.extract_time(&text);
                debug!("GameTime -> {:?} (raw: \"{}\")", time, text);
                Ok(time)
            }
            Err(e) => {
                warn!("OCR failed for GameTime: {}", e);
                Ok(None)
            }
        }
    }
}
