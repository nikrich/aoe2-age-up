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
        cmd.arg("--version");
        if tessdata_path.is_dir() {
            cmd.env("TESSDATA_PREFIX", &tessdata_path);
        }
        cmd.env("PATH", &path_env);
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

        // Encode to PNG and pipe it to tesseract's stdin. A temp file would
        // work too, but a hardcoded path makes concurrent recognitions clobber
        // each other's input.
        let mut png: Vec<u8> = Vec::with_capacity(1 << 16);
        img.write_to(&mut std::io::Cursor::new(&mut png), image::ImageFormat::Png)
            .context("Failed to encode PNG for tesseract")?;

        let path_env = build_path_with_deps(&self.deps_dir);

        let mut cmd = Command::new(&self.tesseract_path);
        cmd.arg("stdin")
            .arg("stdout")
            .arg("--psm").arg("7") // Single line of text
            .arg("-c").arg("tessedit_char_whitelist=0123456789:/")
            .arg("-l").arg("eng")
            .env("PATH", &path_env)
            .stdin(std::process::Stdio::piped())
            .stdout(std::process::Stdio::piped())
            .stderr(std::process::Stdio::piped());
        if self.tessdata_path.is_dir() {
            cmd.env("TESSDATA_PREFIX", &self.tessdata_path);
        }
        #[cfg(target_os = "windows")]
        cmd.creation_flags(CREATE_NO_WINDOW);

        let mut child = cmd.spawn().context("Failed to run tesseract")?;

        {
            use std::io::Write;
            let mut stdin = child.stdin.take().context("Failed to open tesseract stdin")?;
            stdin.write_all(&png).context("Failed to write PNG to tesseract")?;
        }

        let output = child.wait_with_output().context("Failed to run tesseract")?;

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
}

/// Pull a number out of OCR text.
///
/// Reads longer than 5 digits are rejected: they're almost always several
/// neighbouring fields concatenated ("200200"), and since the game state
/// latches the per-field *maximum*, one bogus huge reading would satisfy
/// every resource trigger for the rest of the session.
fn extract_number(text: &str) -> Option<u32> {
    let digits: String = text.chars().filter(|c| c.is_ascii_digit()).collect();
    if digits.is_empty() || digits.len() > 5 {
        return None;
    }
    digits.parse().ok()
}

fn extract_population(text: &str) -> Option<(u32, u32)> {
    let text = text.replace(' ', "");
    let parts: Vec<&str> = text.split('/').collect();
    if parts.len() == 2 {
        let current: u32 = parts[0].chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok()?;
        let max: u32 = parts[1].chars().filter(|c| c.is_ascii_digit()).collect::<String>().parse().ok()?;
        // Sanity cap: AoE2 populations are in the low hundreds at most.
        if current > 999 || max > 999 {
            return None;
        }
        Some((current, max))
    } else {
        None
    }
}

/// Parse `HH:MM:SS` (or `MM:SS`) from OCR text.
///
/// Each segment keeps only its leading digits, so trailing junk that
/// tesseract picks up from the "(Normal game speed)" label — e.g.
/// `00:00:05 (Norm` — still parses as 5 seconds. Segments longer than two
/// digits mean the colons were misread (`00200:05`) and are rejected
/// rather than turned into a huge, permanent timestamp.
fn extract_time(text: &str) -> Option<u32> {
    // Trim stray colons on both ends: tesseract sometimes emits a leading
    // colon when the crop clips the left edge of the clock glyph.
    let text = text.trim().trim_matches(':');
    let seg = |s: &str| -> Option<u32> {
        let d: String = s.trim().chars().take_while(|c| c.is_ascii_digit()).collect();
        if d.is_empty() || d.len() > 2 {
            return None;
        }
        d.parse().ok()
    };
    let parts: Vec<&str> = text.split(':').collect();
    match parts.len() {
        3 => {
            let h = seg(parts[0])?;
            let m = seg(parts[1])?;
            let s = seg(parts[2])?;
            Some(h * 3600 + m * 60 + s)
        }
        2 => {
            let m = seg(parts[0])?;
            let s = seg(parts[1])?;
            Some(m * 60 + s)
        }
        _ => None,
    }
}

/// Prepend the deps directory to the system PATH so tesseract can find its
/// DLLs (Windows) / shared libraries (Linux). Uses the platform's path
/// separator (`;` on Windows, `:` elsewhere). Missing directories are skipped.
fn build_path_with_deps(deps_dir: &std::path::Path) -> String {
    let system_path = std::env::var_os("PATH").unwrap_or_default();
    let mut entries: Vec<std::ffi::OsString> = Vec::new();
    if deps_dir.is_dir() {
        entries.push(deps_dir.as_os_str().to_owned());
    }
    for p in std::env::split_paths(&system_path) {
        entries.push(p.as_os_str().to_owned());
    }
    std::env::join_paths(entries)
        .map(|p| p.to_string_lossy().into_owned())
        .unwrap_or_else(|_| system_path.to_string_lossy().into_owned())
}

impl OcrPipeline for TesseractPipeline {
    fn read_number(&self, image: &[u8], width: u32, height: u32, kind: RegionKind) -> Result<Option<u32>> {
        match self.recognize_text(image, width, height) {
            Ok(text) => {
                let num = extract_number(&text);
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
                let pop = extract_population(&text);
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
                let time = extract_time(&text);
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_reads_plain_and_junk() {
        assert_eq!(extract_number("200"), Some(200));
        assert_eq!(extract_number("  1 234 "), Some(1234));
        assert_eq!(extract_number("no digits"), None);
        assert_eq!(extract_number(""), None);
    }

    #[test]
    fn number_rejects_concatenated_neighbour_fields() {
        // Two fields read as one run — must not become a permanent "peak".
        assert_eq!(extract_number("200200"), None);
        assert_eq!(extract_number("1234567"), None);
    }

    #[test]
    fn population_parsing() {
        assert_eq!(extract_population("4/5"), Some((4, 5)));
        assert_eq!(extract_population(" 4 / 5 "), Some((4, 5)));
        assert_eq!(extract_population("garbage"), None);
        assert_eq!(extract_population("4444/5555"), None);
    }

    #[test]
    fn time_parsing_accepts_trailing_junk() {
        assert_eq!(extract_time("00:00:05"), Some(5));
        // tesseract picks up the "(Normal game speed)" label after the clock
        assert_eq!(extract_time("00:00:05 (Norm"), Some(5));
        // crop clipping the left edge can add a leading colon
        assert_eq!(extract_time(":00:00:08"), Some(8));
        assert_eq!(extract_time("00:00:0:"), Some(0));
        assert_eq!(extract_time("10:30"), Some(630));
        assert_eq!(extract_time("01:02:03"), Some(3723));
    }

    #[test]
    fn time_parsing_rejects_misreads() {
        // Colons misread as digits: no colon-separated structure left
        assert_eq!(extract_time("12012005"), None);
        // A 5-digit segment means the separators weren't colons
        assert_eq!(extract_time("00200:05"), None);
        assert_eq!(extract_time(""), None);
    }
}
