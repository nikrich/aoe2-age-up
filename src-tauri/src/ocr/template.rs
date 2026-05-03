use image::GrayImage;
use image::GenericImageView;
use image::imageops::resize;
use image::imageops::FilterType;
use std::collections::HashMap;

use super::segment::CharBox;

/// Match a character image against templates using normalized cross-correlation.
pub fn match_character(
    binary: &GrayImage,
    char_box: &CharBox,
    templates: &HashMap<char, GrayImage>,
    confidence_threshold: f64,
) -> Option<char> {
    let sub = binary.view(char_box.x, char_box.y, char_box.width, char_box.height)
        .to_image();

    let mut best_char = None;
    let mut best_score = f64::NEG_INFINITY;

    for (&ch, template) in templates {
        let (tw, th) = (template.width(), template.height());
        let resized = resize(&sub, tw, th, FilterType::Nearest);
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
pub fn chars_to_number(chars: &[Option<char>]) -> Option<u32> {
    let s: String = chars.iter().filter_map(|c| *c).filter(|c| c.is_ascii_digit()).collect();
    if s.is_empty() { return None; }
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
        let template_3 = templates.get(&'3').unwrap().clone();
        let char_box = CharBox { x: 0, y: 0, width: template_3.width(), height: template_3.height() };
        let result = match_character(&template_3, &char_box, &templates, 0.7);
        assert_eq!(result, Some('3'));
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
