use aoe_overlay::ocr::fixture::generate_fixture_templates;
use aoe_overlay::ocr::preprocess::{crop_region, to_grayscale, threshold};
use aoe_overlay::ocr::segment::segment_characters;
use aoe_overlay::ocr::template::match_character;
use aoe_overlay::state::{Calibration, RegionKind};

/// Diagnostic test: load a real AoE2:DE screenshot and attempt OCR on each
/// calibrated region. Prints what the pipeline sees at each stage.
#[test]
fn diagnose_ocr_on_real_screenshot() {
    let screenshot_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/aoe.png");

    if !screenshot_path.exists() {
        eprintln!("Screenshot not found at {:?}, skipping", screenshot_path);
        return;
    }

    let img = image::open(&screenshot_path).expect("Failed to open screenshot");
    let rgba = img.to_rgba8();
    let (img_w, img_h) = (rgba.width(), rgba.height());
    eprintln!("Screenshot: {}x{}", img_w, img_h);

    let cal = Calibration::default_1080p();
    let templates = generate_fixture_templates();

    for (kind, region) in &cal.regions {
        eprintln!("\n--- {:?} region: x={}, y={}, w={}, h={} ---",
            kind, region.x, region.y, region.width, region.height);

        let cropped = crop_region(
            rgba.as_raw(),
            img_w,
            img_h,
            region.x,
            region.y,
            region.width,
            region.height,
        );

        let cropped = match cropped {
            Some(c) => c,
            None => {
                eprintln!("  SKIP: crop out of bounds");
                continue;
            }
        };

        // Save cropped region for visual inspection
        let crop_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(format!("docs/crop_{:?}.png", kind));
        let _ = cropped.save(&crop_path);
        eprintln!("  Saved crop to {:?}", crop_path);

        let gray = to_grayscale(&cropped);
        let binary = threshold(&gray, 180);

        // Save binary for inspection
        let bin_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join(format!("docs/binary_{:?}.png", kind));
        let _ = binary.save(&bin_path);

        let boxes = segment_characters(&binary, 2);
        eprintln!("  Segmented {} character boxes", boxes.len());

        for (i, b) in boxes.iter().enumerate() {
            let matched = match_character(&binary, b, &templates, 0.5);
            eprintln!("    Box {}: x={}, y={}, w={}, h={} -> {:?}",
                i, b.x, b.y, b.width, b.height, matched);
        }

        let chars: Vec<Option<char>> = boxes.iter()
            .map(|b| match_character(&binary, b, &templates, 0.5))
            .collect();

        let text: String = chars.iter()
            .map(|c| c.unwrap_or('?'))
            .collect();
        eprintln!("  OCR result: \"{}\"", text);
    }
}
