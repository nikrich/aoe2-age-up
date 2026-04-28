# Phase 2-4: Capture, OCR, Auto-Advance Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the full pipeline from screen capture through OCR to automatic build order step advancement.

**Architecture:** xcap captures full screen at 1Hz, crops to calibrated regions, preprocesses to binary images, segments characters, matches against fixture templates, parses numbers into GameState, evaluates triggers, and auto-advances steps. All runs in a single Tokio task managed by start/stop IPC commands.

**Tech Stack:** Rust (xcap, image, imageproc), Tokio for async capture loop, Tauri IPC events, existing React frontend hooks.

---

## File Structure

### New files
- `src-tauri/src/capture/fallback.rs` — xcap-based CaptureBackend implementation (rewrite stub)
- `src-tauri/src/capture/loop_task.rs` — Tokio capture loop driver
- `src-tauri/src/ocr/preprocess.rs` — grayscale + threshold (rewrite stub)
- `src-tauri/src/ocr/segment.rs` — connected-component character segmentation (rewrite stub)
- `src-tauri/src/ocr/template.rs` — template matching OCR backend (rewrite stub)
- `src-tauri/src/ocr/fixture.rs` — test fixture template generation
- `src-tauri/tests/ocr_pipeline_integration.rs` — end-to-end OCR pipeline test

### Modified files
- `src-tauri/Cargo.toml` — add xcap, image, imageproc dependencies
- `src-tauri/src/capture/mod.rs` — add loop_task module, RGBA format clarification
- `src-tauri/src/ocr/mod.rs` — add fixture module, implement TemplatePipeline
- `src-tauri/src/ipc.rs` — add start_capture, stop_capture commands
- `src-tauri/src/main.rs` — register new commands, add capture task handle to state
- `src-tauri/src/state.rs` — add CaptureHandle for task management
- `src-tauri/src/error.rs` — add CaptureError variant

### Unchanged files (for reference only)
- `src-tauri/src/build_order/engine.rs` — trigger evaluation already complete
- `src/hooks/useGameState.ts` — already listens to "game-state" events
- `src/lib/types.ts` — GameState type already defined

---

## Task 1: Add Dependencies

**Files:**
- Modify: `src-tauri/Cargo.toml`

- [ ] **Step 1: Add xcap, image, imageproc to Cargo.toml**

In `src-tauri/Cargo.toml`, add to `[dependencies]`:

```toml
xcap = "0.0.14"
image = { version = "0.25", default-features = false, features = ["png"] }
imageproc = "0.25"
```

- [ ] **Step 2: Verify it compiles**

Run: `cd src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` with no errors (warnings OK)

- [ ] **Step 3: Commit**

```bash
git add src-tauri/Cargo.toml
git commit -m "feat: add xcap, image, imageproc dependencies for capture and OCR"
```

---

## Task 2: Implement xcap Capture Backend

**Files:**
- Modify: `src-tauri/src/capture/mod.rs`
- Rewrite: `src-tauri/src/capture/fallback.rs`

- [ ] **Step 1: Update CaptureFrame in mod.rs to clarify RGBA format**

Replace the entire content of `src-tauri/src/capture/mod.rs` with:

```rust
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
```

- [ ] **Step 2: Implement XcapCapture in fallback.rs**

Replace the entire content of `src-tauri/src/capture/fallback.rs` with:

```rust
use anyhow::{Context, Result};
use xcap::Monitor;

use super::{CaptureBackend, CaptureFrame};

pub struct XcapCapture {
    monitor: Monitor,
}

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
        // This test requires a display — skip in headless CI
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
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` (loop_task module doesn't exist yet — that's OK, we'll add it next task. If it errors on the missing module, temporarily comment out `pub mod loop_task;` in mod.rs, verify compile, then uncomment.)

Actually — since mod.rs references `loop_task` which doesn't exist yet, create a placeholder:

Create `src-tauri/src/capture/loop_task.rs` with:

```rust
// Capture loop driver — implemented in Task 6
```

- [ ] **Step 4: Run tests**

Run: `cd src-tauri && cargo test capture::fallback --lib 2>&1 | tail -10`
Expected: test passes (or skipped with "no display" message)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/capture/
git commit -m "feat: implement xcap-based screen capture backend"
```

---

## Task 3: Implement Image Preprocessing

**Files:**
- Rewrite: `src-tauri/src/ocr/preprocess.rs`

- [ ] **Step 1: Write tests for grayscale and threshold**

Replace the entire content of `src-tauri/src/ocr/preprocess.rs` with:

```rust
use image::{GrayImage, RgbaImage};

/// Convert an RGBA image to grayscale using luminance formula.
pub fn to_grayscale(rgba: &RgbaImage) -> GrayImage {
    GrayImage::from_fn(rgba.width(), rgba.height(), |x, y| {
        let p = rgba.get_pixel(x, y);
        let lum = (0.299 * p[0] as f32 + 0.587 * p[1] as f32 + 0.114 * p[2] as f32) as u8;
        image::Luma([lum])
    })
}

/// Threshold a grayscale image to binary (0 or 255).
/// Pixels >= threshold become 255 (white), below become 0 (black).
pub fn threshold(gray: &GrayImage, thresh: u8) -> GrayImage {
    GrayImage::from_fn(gray.width(), gray.height(), |x, y| {
        let p = gray.get_pixel(x, y).0[0];
        image::Luma([if p >= thresh { 255 } else { 0 }])
    })
}

/// Crop a region from raw RGBA pixel data and return as RgbaImage.
/// Returns None if the region is out of bounds.
pub fn crop_region(
    data: &[u8],
    frame_width: u32,
    frame_height: u32,
    x: u32,
    y: u32,
    w: u32,
    h: u32,
) -> Option<RgbaImage> {
    if x + w > frame_width || y + h > frame_height {
        return None;
    }
    let mut img = RgbaImage::new(w, h);
    for row in 0..h {
        let src_offset = ((y + row) * frame_width + x) as usize * 4;
        for col in 0..w {
            let px_offset = src_offset + col as usize * 4;
            let pixel = image::Rgba([
                data[px_offset],
                data[px_offset + 1],
                data[px_offset + 2],
                data[px_offset + 3],
            ]);
            img.put_pixel(col, row, pixel);
        }
    }
    Some(img)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grayscale_white_pixel() {
        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, image::Rgba([255, 255, 255, 255]));
        let gray = to_grayscale(&img);
        assert_eq!(gray.get_pixel(0, 0).0[0], 255);
    }

    #[test]
    fn test_grayscale_black_pixel() {
        let mut img = RgbaImage::new(1, 1);
        img.put_pixel(0, 0, image::Rgba([0, 0, 0, 255]));
        let gray = to_grayscale(&img);
        assert_eq!(gray.get_pixel(0, 0).0[0], 0);
    }

    #[test]
    fn test_threshold_above() {
        let mut gray = GrayImage::new(1, 1);
        gray.put_pixel(0, 0, image::Luma([200]));
        let bin = threshold(&gray, 180);
        assert_eq!(bin.get_pixel(0, 0).0[0], 255);
    }

    #[test]
    fn test_threshold_below() {
        let mut gray = GrayImage::new(1, 1);
        gray.put_pixel(0, 0, image::Luma([100]));
        let bin = threshold(&gray, 180);
        assert_eq!(bin.get_pixel(0, 0).0[0], 0);
    }

    #[test]
    fn test_crop_region_basic() {
        // 4x4 RGBA image, all white
        let width = 4u32;
        let height = 4u32;
        let data = vec![255u8; (width * height * 4) as usize];
        let cropped = crop_region(&data, width, height, 1, 1, 2, 2).unwrap();
        assert_eq!(cropped.width(), 2);
        assert_eq!(cropped.height(), 2);
        assert_eq!(cropped.get_pixel(0, 0).0, [255, 255, 255, 255]);
    }

    #[test]
    fn test_crop_region_out_of_bounds() {
        let data = vec![0u8; 16]; // 2x2 RGBA
        assert!(crop_region(&data, 2, 2, 1, 1, 2, 2).is_none());
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd src-tauri && cargo test ocr::preprocess --lib 2>&1 | tail -10`
Expected: 6 tests pass

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/ocr/preprocess.rs
git commit -m "feat: implement image preprocessing (grayscale, threshold, crop)"
```

---

## Task 4: Implement Character Segmentation

**Files:**
- Rewrite: `src-tauri/src/ocr/segment.rs`

- [ ] **Step 1: Implement connected-component segmentation**

Replace the entire content of `src-tauri/src/ocr/segment.rs` with:

```rust
use image::GrayImage;

/// A bounding box for a segmented character.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CharBox {
    pub x: u32,
    pub y: u32,
    pub width: u32,
    pub height: u32,
}

/// Segment a binary image into character bounding boxes, left-to-right.
/// Expects white (255) characters on black (0) background.
/// Uses vertical projection: finds columns with white pixels, groups them.
pub fn segment_characters(binary: &GrayImage, min_width: u32) -> Vec<CharBox> {
    let (w, h) = binary.dimensions();
    if w == 0 || h == 0 {
        return Vec::new();
    }

    // Build column projection: true if column has any white pixel
    let col_has_pixel: Vec<bool> = (0..w)
        .map(|x| (0..h).any(|y| binary.get_pixel(x, y).0[0] > 0))
        .collect();

    // Group consecutive columns with pixels into character regions
    let mut boxes = Vec::new();
    let mut in_char = false;
    let mut start_x = 0u32;

    for (x, &has_pixel) in col_has_pixel.iter().enumerate() {
        if has_pixel && !in_char {
            in_char = true;
            start_x = x as u32;
        } else if !has_pixel && in_char {
            in_char = false;
            let char_w = x as u32 - start_x;
            if char_w >= min_width {
                let (top, bottom) = find_vertical_bounds(binary, start_x, char_w, h);
                boxes.push(CharBox {
                    x: start_x,
                    y: top,
                    width: char_w,
                    height: bottom - top + 1,
                });
            }
        }
    }
    // Handle character at right edge
    if in_char {
        let char_w = w - start_x;
        if char_w >= min_width {
            let (top, bottom) = find_vertical_bounds(binary, start_x, char_w, h);
            boxes.push(CharBox {
                x: start_x,
                y: top,
                width: char_w,
                height: bottom - top + 1,
            });
        }
    }

    boxes
}

fn find_vertical_bounds(img: &GrayImage, start_x: u32, width: u32, img_h: u32) -> (u32, u32) {
    let mut top = img_h;
    let mut bottom = 0u32;
    for y in 0..img_h {
        for x in start_x..start_x + width {
            if img.get_pixel(x, y).0[0] > 0 {
                top = top.min(y);
                bottom = bottom.max(y);
            }
        }
    }
    if top > bottom {
        (0, 0)
    } else {
        (top, bottom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_binary(w: u32, h: u32, white_cols: &[std::ops::Range<u32>]) -> GrayImage {
        let mut img = GrayImage::new(w, h);
        for range in white_cols {
            for x in range.clone() {
                for y in 0..h {
                    img.put_pixel(x, y, image::Luma([255]));
                }
            }
        }
        img
    }

    #[test]
    fn test_empty_image() {
        let img = GrayImage::new(10, 10);
        let boxes = segment_characters(&img, 1);
        assert!(boxes.is_empty());
    }

    #[test]
    fn test_single_character() {
        // White columns 2..5 (3 wide)
        let img = make_binary(10, 10, &[2..5]);
        let boxes = segment_characters(&img, 1);
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].x, 2);
        assert_eq!(boxes[0].width, 3);
    }

    #[test]
    fn test_two_characters_with_gap() {
        // Two groups: cols 1..3 and cols 6..9
        let img = make_binary(12, 8, &[1..3, 6..9]);
        let boxes = segment_characters(&img, 1);
        assert_eq!(boxes.len(), 2);
        assert_eq!(boxes[0].x, 1);
        assert_eq!(boxes[1].x, 6);
    }

    #[test]
    fn test_min_width_filters_noise() {
        // 1-pixel wide "noise" at col 5
        let img = make_binary(10, 10, &[5..6]);
        let boxes = segment_characters(&img, 2);
        assert!(boxes.is_empty());
    }

    #[test]
    fn test_character_at_right_edge() {
        let img = make_binary(10, 8, &[7..10]);
        let boxes = segment_characters(&img, 1);
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].x, 7);
        assert_eq!(boxes[0].width, 3);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cd src-tauri && cargo test ocr::segment --lib 2>&1 | tail -10`
Expected: 5 tests pass

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/ocr/segment.rs
git commit -m "feat: implement character segmentation via vertical projection"
```

---

## Task 5: Implement Template Matching and OCR Pipeline

**Files:**
- Rewrite: `src-tauri/src/ocr/template.rs`
- Create: `src-tauri/src/ocr/fixture.rs`
- Modify: `src-tauri/src/ocr/mod.rs`

- [ ] **Step 1: Create fixture template generator**

Create `src-tauri/src/ocr/fixture.rs`:

```rust
use image::GrayImage;
use std::collections::HashMap;

/// Generate simple synthetic digit templates for testing.
/// Each template is a 10x14 binary image with a crude digit pattern.
/// These are NOT for production use — replace with real AoE2 font templates.
pub fn generate_fixture_templates() -> HashMap<char, GrayImage> {
    let mut templates = HashMap::new();
    // Simple 5-wide bitmaps for digits 0-9, scaled to 10x14
    let patterns: [(char, [u8; 7]); 12] = [
        ('0', [0b01110, 0b10001, 0b10001, 0b10001, 0b10001, 0b10001, 0b01110]),
        ('1', [0b00100, 0b01100, 0b00100, 0b00100, 0b00100, 0b00100, 0b01110]),
        ('2', [0b01110, 0b10001, 0b00001, 0b00110, 0b01000, 0b10000, 0b11111]),
        ('3', [0b01110, 0b10001, 0b00001, 0b00110, 0b00001, 0b10001, 0b01110]),
        ('4', [0b00010, 0b00110, 0b01010, 0b10010, 0b11111, 0b00010, 0b00010]),
        ('5', [0b11111, 0b10000, 0b11110, 0b00001, 0b00001, 0b10001, 0b01110]),
        ('6', [0b01110, 0b10000, 0b10000, 0b11110, 0b10001, 0b10001, 0b01110]),
        ('7', [0b11111, 0b00001, 0b00010, 0b00100, 0b00100, 0b00100, 0b00100]),
        ('8', [0b01110, 0b10001, 0b10001, 0b01110, 0b10001, 0b10001, 0b01110]),
        ('9', [0b01110, 0b10001, 0b10001, 0b01111, 0b00001, 0b00001, 0b01110]),
        ('/', [0b00001, 0b00010, 0b00010, 0b00100, 0b01000, 0b01000, 0b10000]),
        (':', [0b00000, 0b00100, 0b00100, 0b00000, 0b00100, 0b00100, 0b00000]),
    ];

    for (ch, rows) in &patterns {
        let tw = 10u32;
        let th = 14u32;
        let mut img = GrayImage::new(tw, th);
        for (src_y, &row_bits) in rows.iter().enumerate() {
            let dst_y0 = src_y as u32 * 2;
            for src_x in 0..5u32 {
                let bit = (row_bits >> (4 - src_x)) & 1;
                let val = if bit == 1 { 255u8 } else { 0u8 };
                let dst_x0 = src_x * 2;
                // 2x2 scaling
                for dy in 0..2u32 {
                    for dx in 0..2u32 {
                        if dst_y0 + dy < th && dst_x0 + dx < tw {
                            img.put_pixel(dst_x0 + dx, dst_y0 + dy, image::Luma([val]));
                        }
                    }
                }
            }
        }
        templates.insert(*ch, img);
    }

    templates
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generates_all_templates() {
        let templates = generate_fixture_templates();
        assert_eq!(templates.len(), 12);
        for ch in "0123456789/:".chars() {
            assert!(templates.contains_key(&ch), "missing template for '{}'", ch);
        }
    }

    #[test]
    fn test_template_dimensions() {
        let templates = generate_fixture_templates();
        for (ch, img) in &templates {
            assert_eq!(img.width(), 10, "wrong width for '{}'", ch);
            assert_eq!(img.height(), 14, "wrong height for '{}'", ch);
        }
    }

    #[test]
    fn test_templates_have_white_pixels() {
        let templates = generate_fixture_templates();
        for (ch, img) in &templates {
            let has_white = img.pixels().any(|p| p.0[0] > 0);
            assert!(has_white, "template '{}' has no white pixels", ch);
        }
    }
}
```

- [ ] **Step 2: Implement template matching in template.rs**

Replace the entire content of `src-tauri/src/ocr/template.rs` with:

```rust
use image::GrayImage;
use image::imageops::resize;
use image::imageops::FilterType;
use std::collections::HashMap;

use super::segment::CharBox;

/// Match a character image against templates using normalized cross-correlation.
/// Returns the best matching character if confidence exceeds threshold.
pub fn match_character(
    binary: &GrayImage,
    char_box: &CharBox,
    templates: &HashMap<char, GrayImage>,
    confidence_threshold: f64,
) -> Option<char> {
    let sub = binary.view(char_box.x, char_box.y, char_box.width, char_box.height)
        .to_image();

    // Get template dimensions from any template
    let (tw, th) = templates.values().next().map(|t| (t.width(), t.height()))?;

    // Resize extracted character to template dimensions
    let resized = resize(&sub, tw, th, FilterType::Nearest);

    let mut best_char = None;
    let mut best_score = f64::NEG_INFINITY;

    for (&ch, template) in templates {
        let score = normalized_cross_correlation(&resized, template);
        if score > best_score {
            best_score = score;
            best_char = Some(ch);
        }
    }

    if best_score >= confidence_threshold {
        best_char
    } else {
        None
    }
}

/// Normalized cross-correlation between two same-size grayscale images.
/// Returns a value in [-1, 1] where 1 = perfect match.
fn normalized_cross_correlation(a: &GrayImage, b: &GrayImage) -> f64 {
    assert_eq!(a.dimensions(), b.dimensions());

    let n = (a.width() * a.height()) as f64;
    let mut sum_a = 0.0f64;
    let mut sum_b = 0.0f64;
    let mut sum_aa = 0.0f64;
    let mut sum_bb = 0.0f64;
    let mut sum_ab = 0.0f64;

    for (pa, pb) in a.pixels().zip(b.pixels()) {
        let va = pa.0[0] as f64;
        let vb = pb.0[0] as f64;
        sum_a += va;
        sum_b += vb;
        sum_aa += va * va;
        sum_bb += vb * vb;
        sum_ab += va * vb;
    }

    let mean_a = sum_a / n;
    let mean_b = sum_b / n;
    let var_a = (sum_aa / n - mean_a * mean_a).max(0.0);
    let var_b = (sum_bb / n - mean_b * mean_b).max(0.0);
    let std_a = var_a.sqrt();
    let std_b = var_b.sqrt();

    if std_a < 1e-10 || std_b < 1e-10 {
        return 0.0;
    }

    (sum_ab / n - mean_a * mean_b) / (std_a * std_b)
}

/// Parse a sequence of matched characters into a u32.
/// Handles pure digits like "1234" and ignores unrecognized chars.
pub fn chars_to_number(chars: &[Option<char>]) -> Option<u32> {
    let s: String = chars.iter().filter_map(|c| *c).filter(|c| c.is_ascii_digit()).collect();
    if s.is_empty() {
        return None;
    }
    s.parse().ok()
}

/// Parse a time string like "12:34" into total seconds.
pub fn chars_to_time_seconds(chars: &[Option<char>]) -> Option<u32> {
    let s: String = chars.iter().filter_map(|c| *c).collect();
    let parts: Vec<&str> = s.split(':').collect();
    match parts.len() {
        2 => {
            let mins: u32 = parts[0].parse().ok()?;
            let secs: u32 = parts[1].parse().ok()?;
            Some(mins * 60 + secs)
        }
        1 => parts[0].parse().ok(),
        _ => None,
    }
}

/// Parse a population string like "45/200" into (current, max).
pub fn chars_to_population(chars: &[Option<char>]) -> Option<(u32, u32)> {
    let s: String = chars.iter().filter_map(|c| *c).collect();
    let parts: Vec<&str> = s.split('/').collect();
    if parts.len() == 2 {
        let current: u32 = parts[0].parse().ok()?;
        let max: u32 = parts[1].parse().ok()?;
        Some((current, max))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ocr::fixture::generate_fixture_templates;

    #[test]
    fn test_ncc_identical_images() {
        let mut a = GrayImage::new(10, 10);
        for y in 0..10 {
            for x in 0..10 {
                a.put_pixel(x, y, image::Luma([((x + y * 10) % 256) as u8]));
            }
        }
        let score = normalized_cross_correlation(&a, &a);
        assert!((score - 1.0).abs() < 1e-6, "NCC of identical images should be ~1.0, got {}", score);
    }

    #[test]
    fn test_match_character_with_fixture_templates() {
        let templates = generate_fixture_templates();
        // Create a test image from the '3' template
        let template_3 = templates.get(&'3').unwrap().clone();
        let char_box = CharBox { x: 0, y: 0, width: template_3.width(), height: template_3.height() };
        let result = match_character(&template_3, &char_box, &templates, 0.7);
        assert_eq!(result, Some('3'), "should match '3' template against itself");
    }

    #[test]
    fn test_chars_to_number() {
        assert_eq!(chars_to_number(&[Some('1'), Some('2'), Some('3')]), Some(123));
        assert_eq!(chars_to_number(&[Some('0')]), Some(0));
        assert_eq!(chars_to_number(&[None, Some('5')]), Some(5));
        assert_eq!(chars_to_number(&[None]), None);
    }

    #[test]
    fn test_chars_to_time_seconds() {
        assert_eq!(chars_to_time_seconds(&[Some('1'), Some('2'), Some(':'), Some('3'), Some('0')]), Some(750));
        assert_eq!(chars_to_time_seconds(&[Some('0'), Some(':'), Some('0'), Some('0')]), Some(0));
    }

    #[test]
    fn test_chars_to_population() {
        assert_eq!(chars_to_population(&[Some('4'), Some('5'), Some('/'), Some('2'), Some('0'), Some('0')]), Some((45, 200)));
        assert_eq!(chars_to_population(&[Some('5')]), None);
    }
}
```

- [ ] **Step 3: Update ocr/mod.rs with TemplatePipeline**

Replace the entire content of `src-tauri/src/ocr/mod.rs` with:

```rust
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
            threshold_value: 180,
            confidence: 0.7,
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
```

- [ ] **Step 4: Run all OCR tests**

Run: `cd src-tauri && cargo test ocr --lib 2>&1 | tail -15`
Expected: All tests pass (fixture: 3, preprocess: 6, segment: 5, template: 5 = 19 total)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/ocr/
git commit -m "feat: implement template matching OCR pipeline with fixture templates"
```

---

## Task 6: Implement Capture Loop

**Files:**
- Rewrite: `src-tauri/src/capture/loop_task.rs`
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: Add CaptureHandle to state.rs**

Add the following after the `AppState` struct in `src-tauri/src/state.rs`:

```rust
use tokio::sync::oneshot;

/// Handle for controlling the capture loop from IPC commands.
pub struct CaptureHandle {
    pub stop_tx: Option<oneshot::Sender<()>>,
}
```

Note: `CaptureHandle` is NOT part of `AppState` (which is behind Mutex). It will be managed separately by Tauri.

- [ ] **Step 2: Implement capture loop driver**

Replace the entire content of `src-tauri/src/capture/loop_task.rs` with:

```rust
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Instant;

use anyhow::Result;
use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::oneshot;
use tracing::{info, warn};

use crate::build_order::engine::evaluate;
use crate::ipc::emit_step_changed;
use crate::ocr::preprocess::crop_region;
use crate::ocr::OcrPipeline;
use crate::state::{AppState, GameState, RegionKind};

use super::CaptureBackend;

/// Spawn the capture loop as a Tokio task. Returns a stop sender.
pub fn spawn_capture_loop(
    app: AppHandle,
    mut backend: Box<dyn CaptureBackend>,
    ocr: Box<dyn OcrPipeline>,
) -> oneshot::Sender<()> {
    let (stop_tx, mut stop_rx) = oneshot::channel::<()>();

    tokio::spawn(async move {
        info!("Capture loop started");
        let mut last_hashes: HashMap<RegionKind, u64> = HashMap::new();

        loop {
            // Check for stop signal
            if stop_rx.try_recv().is_ok() {
                info!("Capture loop received stop signal");
                break;
            }

            let interval_ms = {
                let state_mutex = app.state::<Mutex<AppState>>();
                let s = state_mutex.lock().unwrap();
                if !s.capture_running {
                    break;
                }
                s.settings.capture_interval_ms
            };

            // Capture frame (blocking — run in spawn_blocking)
            let frame = {
                let result = tokio::task::spawn_blocking(move || {
                    let frame = backend.next_frame();
                    (backend, frame)
                })
                .await;

                match result {
                    Ok((b, Ok(frame))) => {
                        backend = b;
                        frame
                    }
                    Ok((b, Err(e))) => {
                        backend = b;
                        warn!("Capture failed: {}", e);
                        tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms)).await;
                        continue;
                    }
                    Err(e) => {
                        warn!("Capture task panicked: {}", e);
                        break;
                    }
                }
            };

            // Read calibration regions and process OCR
            let regions = {
                let state_mutex = app.state::<Mutex<AppState>>();
                let s = state_mutex.lock().unwrap();
                s.calibration.regions.clone()
            };

            let mut new_game_state = GameState::default();
            let mut any_update = false;

            for (kind, region) in &regions {
                let cropped = crop_region(
                    &frame.data,
                    frame.width,
                    frame.height,
                    region.x,
                    region.y,
                    region.width,
                    region.height,
                );

                let cropped = match cropped {
                    Some(c) => c,
                    None => continue,
                };

                // Quick hash to skip unchanged regions
                let hash = quick_hash(cropped.as_raw());
                if last_hashes.get(kind) == Some(&hash) {
                    continue;
                }
                last_hashes.insert(*kind, hash);
                any_update = true;

                let raw = cropped.as_raw();
                let w = cropped.width();
                let h = cropped.height();

                match kind {
                    RegionKind::Food => {
                        new_game_state.food = ocr.read_number(raw, w, h, *kind).unwrap_or(None);
                    }
                    RegionKind::Wood => {
                        new_game_state.wood = ocr.read_number(raw, w, h, *kind).unwrap_or(None);
                    }
                    RegionKind::Gold => {
                        new_game_state.gold = ocr.read_number(raw, w, h, *kind).unwrap_or(None);
                    }
                    RegionKind::Stone => {
                        new_game_state.stone = ocr.read_number(raw, w, h, *kind).unwrap_or(None);
                    }
                    RegionKind::Villagers => {
                        new_game_state.villagers = ocr.read_number(raw, w, h, *kind).unwrap_or(None);
                    }
                    RegionKind::Population => {
                        new_game_state.population = ocr.read_population(raw, w, h).unwrap_or(None);
                    }
                    RegionKind::GameTime => {
                        new_game_state.game_time_seconds = ocr.read_time(raw, w, h).unwrap_or(None);
                    }
                }
            }

            if any_update {
                new_game_state.last_updated = Some(Instant::now());

                // Emit game state to frontend
                let _ = app.emit("game-state", &new_game_state);

                // Evaluate triggers and auto-advance
                let state_mutex = app.state::<Mutex<AppState>>();
                let mut s = state_mutex.lock().unwrap();
                s.last_game_state = new_game_state.clone();

                if s.settings.auto_advance {
                    if let Some(ref bo) = s.current_build_order {
                        if s.current_step_index < bo.steps.len() - 1 {
                            let next_trigger = &bo.steps[s.current_step_index + 1].at;
                            if evaluate(next_trigger, &new_game_state) {
                                s.current_step_index += 1;
                                emit_step_changed(&app, &s);
                                info!("Auto-advanced to step {}", s.current_step_index);
                            }
                        }
                    }
                }
            }

            tokio::time::sleep(tokio::time::Duration::from_millis(interval_ms)).await;
        }

        info!("Capture loop stopped");
    });

    stop_tx
}

fn quick_hash(data: &[u8]) -> u64 {
    // FNV-1a hash for fast comparison
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data.iter().step_by(16) {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
```

- [ ] **Step 3: Verify it compiles**

Run: `cd src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` with no errors

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/capture/loop_task.rs src-tauri/src/state.rs
git commit -m "feat: implement capture loop with OCR processing and auto-advance"
```

---

## Task 7: Wire IPC Commands and App Setup

**Files:**
- Modify: `src-tauri/src/ipc.rs`
- Modify: `src-tauri/src/error.rs`
- Modify: `src-tauri/src/main.rs`

- [ ] **Step 1: Add CaptureError to error.rs**

In `src-tauri/src/error.rs`, add a new variant to `AppError`:

```rust
    #[error("Capture error: {0}")]
    Capture(String),
```

- [ ] **Step 2: Add start_capture and stop_capture commands to ipc.rs**

Add these imports at the top of `src-tauri/src/ipc.rs`:

```rust
use tokio::sync::oneshot;
use crate::capture::fallback::XcapCapture;
use crate::capture::loop_task::spawn_capture_loop;
use crate::ocr::{TemplatePipeline, fixture::generate_fixture_templates};
use crate::state::CaptureHandle;
```

Add these commands at the bottom of `src-tauri/src/ipc.rs` (before the closing of the file):

```rust
#[tauri::command]
pub fn start_capture(
    app: AppHandle,
    state: State<Mutex<AppState>>,
    capture_handle: State<Mutex<CaptureHandle>>,
) -> Result<(), AppError> {
    {
        let mut s = state.lock().unwrap();
        if s.capture_running {
            return Ok(()); // Already running
        }
        s.capture_running = true;
    }

    let backend = Box::new(
        XcapCapture::new().map_err(|e| AppError::Capture(e.to_string()))?
    );
    let templates = generate_fixture_templates();
    let ocr = Box::new(TemplatePipeline::new(templates));

    let stop_tx = spawn_capture_loop(app, backend, ocr);

    {
        let mut handle = capture_handle.lock().unwrap();
        handle.stop_tx = Some(stop_tx);
    }

    Ok(())
}

#[tauri::command]
pub fn stop_capture(
    state: State<Mutex<AppState>>,
    capture_handle: State<Mutex<CaptureHandle>>,
) -> Result<(), AppError> {
    {
        let mut s = state.lock().unwrap();
        s.capture_running = false;
    }

    let mut handle = capture_handle.lock().unwrap();
    if let Some(tx) = handle.stop_tx.take() {
        let _ = tx.send(());
    }

    Ok(())
}
```

- [ ] **Step 3: Update main.rs to register new commands and manage CaptureHandle**

Replace the entire content of `src-tauri/src/main.rs` with:

```rust
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::sync::Mutex;
use tauri::Manager;

mod build_order;
mod capture;
mod error;
mod hotkeys;
mod ipc;
mod ocr;
mod state;
mod storage;

use state::{AppState, CaptureHandle};
use storage::Storage;

fn main() {
    tracing_subscriber::fmt::init();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .setup(|app| {
            let app_data_dir = app.path().app_data_dir().expect("Failed to get app data dir");
            let storage = Storage::new(app_data_dir)?;

            let bundled_dir = app
                .path()
                .resource_dir()
                .expect("Failed to get resource dir")
                .join("build-orders");
            let _ = storage.copy_bundled_build_orders(&bundled_dir);

            let settings = storage.load_settings().unwrap_or_default();
            let app_state = AppState {
                settings,
                ..AppState::default()
            };

            app.manage(Mutex::new(app_state));
            app.manage(Mutex::new(CaptureHandle { stop_tx: None }));
            app.manage(storage);

            hotkeys::setup_hotkeys(&app.handle().clone());

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            ipc::load_build_order_cmd,
            ipc::list_build_orders_cmd,
            ipc::advance_step,
            ipc::previous_step,
            ipc::reset_steps,
            ipc::get_settings,
            ipc::get_calibration,
            ipc::start_capture,
            ipc::stop_capture,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- [ ] **Step 4: Verify it compiles**

Run: `cd src-tauri && cargo check 2>&1 | tail -5`
Expected: `Finished` with no errors

- [ ] **Step 5: Run all tests**

Run: `cd src-tauri && cargo test 2>&1 | tail -15`
Expected: All existing tests (16 unit + 2 integration) plus new tests (~19 OCR + capture) pass

- [ ] **Step 6: Commit**

```bash
git add src-tauri/src/ipc.rs src-tauri/src/error.rs src-tauri/src/main.rs
git commit -m "feat: wire start/stop capture IPC commands and app setup"
```

---

## Task 8: Add Default Calibration Profile

**Files:**
- Modify: `src-tauri/src/state.rs`

- [ ] **Step 1: Add default_1080p calibration to Calibration**

In `src-tauri/src/state.rs`, replace the `impl Default for Calibration` block with:

```rust
impl Calibration {
    /// Default calibration for AoE2:DE at 1920x1080 resolution.
    /// These are approximate regions for the standard UI layout.
    pub fn default_1080p() -> Self {
        let mut regions = HashMap::new();
        // Resource bar positions (approximate for 1080p AoE2:DE)
        regions.insert(RegionKind::Food, Region { x: 68, y: 0, width: 50, height: 22 });
        regions.insert(RegionKind::Wood, Region { x: 175, y: 0, width: 50, height: 22 });
        regions.insert(RegionKind::Gold, Region { x: 282, y: 0, width: 50, height: 22 });
        regions.insert(RegionKind::Stone, Region { x: 389, y: 0, width: 50, height: 22 });
        regions.insert(RegionKind::Population, Region { x: 500, y: 0, width: 70, height: 22 });
        regions.insert(RegionKind::GameTime, Region { x: 910, y: 0, width: 60, height: 22 });
        regions.insert(RegionKind::Villagers, Region { x: 600, y: 0, width: 30, height: 22 });

        Self {
            profile_name: "1080p-default".to_string(),
            resolution: (1920, 1080),
            ui_scale: 1.0,
            regions,
        }
    }
}

impl Default for Calibration {
    fn default() -> Self {
        Self::default_1080p()
    }
}
```

- [ ] **Step 2: Verify it compiles and tests pass**

Run: `cd src-tauri && cargo test 2>&1 | tail -10`
Expected: All tests pass

- [ ] **Step 3: Commit**

```bash
git add src-tauri/src/state.rs
git commit -m "feat: add default 1080p calibration profile for AoE2:DE"
```

---

## Task 9: Add Frontend Capture Controls

**Files:**
- Modify: `src/components/Overlay.tsx`
- Modify: `src/lib/types.ts`

- [ ] **Step 1: Add CaptureState type to types.ts**

Add at the bottom of `src/lib/types.ts`:

```typescript
export interface Calibration {
  profile_name: string;
  resolution: [number, number];
  ui_scale: number;
  regions: Record<string, Region>;
}

export interface Region {
  x: number;
  y: number;
  width: number;
  height: number;
}
```

- [ ] **Step 2: Add capture controls to Overlay.tsx**

Replace the entire content of `src/components/Overlay.tsx` with:

```tsx
import { useState } from "react";
import { invoke } from "@tauri-apps/api/core";
import { useBuildOrder } from "../hooks/useBuildOrder";
import { useGameState } from "../hooks/useGameState";
import { StepCard } from "./StepCard";

interface OverlayProps {
  onOpenLibrary: () => void;
}

export function Overlay({ onOpenLibrary }: OverlayProps) {
  const { buildOrder, currentStep, totalSteps, advance, previous, reset } = useBuildOrder();
  const gameState = useGameState();
  const [capturing, setCapturing] = useState(false);

  const toggleCapture = async () => {
    try {
      if (capturing) {
        await invoke("stop_capture");
        setCapturing(false);
      } else {
        await invoke("start_capture");
        setCapturing(true);
      }
    } catch (e) {
      console.error("Capture toggle failed:", e);
    }
  };

  if (!buildOrder) {
    return (
      <div className="overlay">
        <div className="no-build-order">
          <div>No build order loaded</div>
          <button className="nav-btn" onClick={onOpenLibrary}>Open Library</button>
        </div>
      </div>
    );
  }

  const currentStepData = buildOrder.steps[currentStep];
  const nextStepData = currentStep < buildOrder.steps.length - 1 ? buildOrder.steps[currentStep + 1] : null;

  return (
    <div className="overlay">
      <div className="overlay-header">
        <span className="overlay-title">{buildOrder.name}</span>
        <span className="step-counter">{currentStep + 1} / {totalSteps}</span>
      </div>
      {currentStepData && <StepCard step={currentStepData} variant="current" />}
      {nextStepData && <StepCard step={nextStepData} variant="next" />}
      {gameState && (
        <div className="game-state-bar">
          {gameState.food != null && <span className="resource food">F:{gameState.food}</span>}
          {gameState.wood != null && <span className="resource wood">W:{gameState.wood}</span>}
          {gameState.gold != null && <span className="resource gold">G:{gameState.gold}</span>}
          {gameState.stone != null && <span className="resource stone">S:{gameState.stone}</span>}
          {gameState.villagers != null && <span className="resource vils">V:{gameState.villagers}</span>}
          {gameState.game_time_seconds != null && (
            <span className="resource time">
              {Math.floor(gameState.game_time_seconds / 60)}:{String(gameState.game_time_seconds % 60).padStart(2, "0")}
            </span>
          )}
        </div>
      )}
      <div className="overlay-nav">
        <button className="nav-btn" onClick={previous}>Prev</button>
        <button className="nav-btn" onClick={reset}>Reset</button>
        <button className="nav-btn" onClick={advance}>Next</button>
        <button className={`nav-btn ${capturing ? "nav-btn--active" : ""}`} onClick={toggleCapture}>
          {capturing ? "Stop" : "Capture"}
        </button>
      </div>
    </div>
  );
}
```

- [ ] **Step 3: Verify frontend builds**

Run: `cd E:/Development/open-age && npm run build 2>&1 | tail -5`
Expected: Build succeeds

- [ ] **Step 4: Commit**

```bash
git add src/components/Overlay.tsx src/lib/types.ts
git commit -m "feat: add capture controls and game state display to overlay"
```

---

## Task 10: Integration Test and Final Verification

**Files:**
- Create: `src-tauri/tests/ocr_pipeline_integration.rs`

- [ ] **Step 1: Write end-to-end OCR pipeline test**

Create `src-tauri/tests/ocr_pipeline_integration.rs`:

```rust
use aoe_overlay::ocr::fixture::generate_fixture_templates;
use aoe_overlay::ocr::preprocess::{to_grayscale, threshold};
use aoe_overlay::ocr::segment::segment_characters;
use aoe_overlay::ocr::template::{match_character, chars_to_number};

/// Render a sequence of digits into a binary image using fixture templates,
/// then verify the OCR pipeline reads them back correctly.
#[test]
fn test_end_to_end_digit_recognition() {
    let templates = generate_fixture_templates();
    let digits = ['1', '2', '3'];

    // Compose an image by placing templates side by side with 2px gaps
    let tw = 10u32;
    let th = 14u32;
    let gap = 2u32;
    let total_w = digits.len() as u32 * tw + (digits.len() as u32 - 1) * gap;
    let mut composed = image::RgbaImage::new(total_w, th);

    // Fill black
    for pixel in composed.pixels_mut() {
        *pixel = image::Rgba([0, 0, 0, 255]);
    }

    // Place each digit template
    for (i, ch) in digits.iter().enumerate() {
        let template = templates.get(ch).unwrap();
        let x_offset = i as u32 * (tw + gap);
        for y in 0..th {
            for x in 0..tw {
                let val = template.get_pixel(x, y).0[0];
                composed.put_pixel(x_offset + x, y, image::Rgba([val, val, val, 255]));
            }
        }
    }

    // Run through pipeline: grayscale -> threshold -> segment -> match
    let gray = to_grayscale(&composed);
    let binary = threshold(&gray, 128);
    let boxes = segment_characters(&binary, 2);

    assert_eq!(boxes.len(), 3, "Expected 3 character boxes, got {}", boxes.len());

    let matched: Vec<Option<char>> = boxes
        .iter()
        .map(|b| match_character(&binary, b, &templates, 0.7))
        .collect();

    assert_eq!(matched, vec![Some('1'), Some('2'), Some('3')]);

    let number = chars_to_number(&matched);
    assert_eq!(number, Some(123));
}

/// Test that a single digit round-trips correctly for all 10 digits.
#[test]
fn test_all_digit_round_trips() {
    let templates = generate_fixture_templates();

    for digit in '0'..='9' {
        let template = templates.get(&digit).unwrap();
        let tw = template.width();
        let th = template.height();

        // Create RGBA image from template
        let mut rgba = image::RgbaImage::new(tw, th);
        for y in 0..th {
            for x in 0..tw {
                let val = template.get_pixel(x, y).0[0];
                rgba.put_pixel(x, y, image::Rgba([val, val, val, 255]));
            }
        }

        let gray = to_grayscale(&rgba);
        let binary = threshold(&gray, 128);
        let boxes = segment_characters(&binary, 2);

        assert!(!boxes.is_empty(), "No boxes found for digit '{}'", digit);

        let matched = match_character(&binary, &boxes[0], &templates, 0.7);
        assert_eq!(
            matched,
            Some(digit),
            "Digit '{}' did not round-trip correctly, got {:?}",
            digit,
            matched
        );
    }
}
```

- [ ] **Step 2: Run integration tests**

Run: `cd src-tauri && cargo test --test ocr_pipeline_integration 2>&1 | tail -10`
Expected: 2 tests pass

- [ ] **Step 3: Run ALL tests (unit + integration)**

Run: `cd src-tauri && cargo test 2>&1 | tail -20`
Expected: All tests pass — original 18 + new ~21 = ~39 tests total

- [ ] **Step 4: Verify full application builds**

Run: `cd E:/Development/open-age && export PATH="/c/Users/janni/.cargo/bin:$PATH" && cargo tauri build 2>&1 | tail -10`

(If this is too slow, at minimum verify `cargo check` and `npm run build` both pass.)

- [ ] **Step 5: Commit**

```bash
git add src-tauri/tests/ocr_pipeline_integration.rs
git commit -m "test: add end-to-end OCR pipeline integration tests"
```

---

## Task 11: Clean Up Stubs and Final Commit

**Files:**
- Modify: `src-tauri/src/capture/windows.rs`
- Modify: `src-tauri/src/ocr/tesseract.rs`

- [ ] **Step 1: Update windows.rs stub with proper TODO**

Replace the content of `src-tauri/src/capture/windows.rs` with:

```rust
// Windows.Graphics.Capture implementation
// TODO: Implement using the `windows` crate for hardware-accelerated,
// region-only capture. This will replace xcap for better performance.
// See spec.md §4.1 for implementation details.
```

- [ ] **Step 2: Update tesseract.rs stub with proper TODO**

Replace the content of `src-tauri/src/ocr/tesseract.rs` with:

```rust
// Tesseract OCR backend (feature-gated behind `tesseract` feature)
// TODO: Implement using `rusty-tesseract` crate with:
// - tessedit_char_whitelist = "0123456789/:"
// - psm = 7 (single text line)
// - oem = 3 (default LSTM engine)
// See spec.md §5.3 for implementation details.
```

- [ ] **Step 3: Final full test run**

Run: `cd src-tauri && cargo test 2>&1`
Expected: All tests pass

- [ ] **Step 4: Commit**

```bash
git add src-tauri/src/capture/windows.rs src-tauri/src/ocr/tesseract.rs
git commit -m "docs: update stub files with implementation notes"
```

- [ ] **Step 5: Push branch and create PR**

```bash
git push -u origin feat/phase-2-capture
```

Then create a PR to main.
