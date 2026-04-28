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

    // Compose an image by placing templates side by side with 2px gaps.
    // Each template is placed in a cell that includes the full template width
    // so that segmentation produces bounding boxes matching template dimensions.
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

    // Use a lower confidence threshold to account for resize distortion
    // when the segmented bounding box differs from template dimensions.
    let matched: Vec<Option<char>> = boxes
        .iter()
        .map(|b| match_character(&binary, b, &templates, 0.5))
        .collect();

    assert_eq!(matched, vec![Some('1'), Some('2'), Some('3')]);

    let number = chars_to_number(&matched);
    assert_eq!(number, Some(123));
}

/// Test that a single digit round-trips correctly for all 10 digits.
///
/// Each digit template is placed centered in a canvas slightly wider than
/// the template, ensuring segmentation captures the character region
/// which then gets resized back to template size for matching.
#[test]
fn test_all_digit_round_trips() {
    let templates = generate_fixture_templates();

    for digit in '0'..='9' {
        let template = templates.get(&digit).unwrap();
        let tw = template.width();
        let th = template.height();

        // Place the template in a canvas with 1px black border on each side
        // so segmentation produces a box that, when resized, closely matches
        // the original template.
        let pad = 1u32;
        let canvas_w = tw + pad * 2;
        let mut rgba = image::RgbaImage::new(canvas_w, th);

        // Fill black
        for pixel in rgba.pixels_mut() {
            *pixel = image::Rgba([0, 0, 0, 255]);
        }

        // Place template centered
        for y in 0..th {
            for x in 0..tw {
                let val = template.get_pixel(x, y).0[0];
                rgba.put_pixel(pad + x, y, image::Rgba([val, val, val, 255]));
            }
        }

        let gray = to_grayscale(&rgba);
        let binary = threshold(&gray, 128);
        let boxes = segment_characters(&binary, 2);

        assert!(!boxes.is_empty(), "No boxes found for digit '{}'", digit);

        let matched = match_character(&binary, &boxes[0], &templates, 0.5);
        assert_eq!(
            matched,
            Some(digit),
            "Digit '{}' did not round-trip correctly, got {:?}",
            digit,
            matched
        );
    }
}
