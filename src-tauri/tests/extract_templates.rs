use aoe_overlay::ocr::preprocess::{crop_region, to_grayscale, threshold};
use aoe_overlay::ocr::segment::segment_characters;
use aoe_overlay::state::Calibration;
use aoe_overlay::state::RegionKind;

/// Extract real digit templates from a known AoE2:DE screenshot.
/// The screenshot at docs/aoe.png has known values:
///   Food=200, Wood=200, Gold=100, Stone=200, Pop=4/5, Time=00:00:05
#[test]
fn extract_real_templates() {
    let screenshot_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/aoe.png");

    if !screenshot_path.exists() {
        eprintln!("Screenshot not found, skipping");
        return;
    }

    let img = image::open(&screenshot_path).expect("Failed to open screenshot");
    let rgba = img.to_rgba8();
    let (img_w, img_h) = (rgba.width(), rgba.height());

    let cal = Calibration::default_1080p();
    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().join("docs");

    // Known region values for labeling
    let regions_with_labels: Vec<(RegionKind, &str)> = vec![
        (RegionKind::Food, "200"),
        (RegionKind::Wood, "200"),
        (RegionKind::Gold, "100"),
        (RegionKind::Stone, "200"),
        (RegionKind::Population, "4/5"),
        (RegionKind::GameTime, "00:00:05"),
    ];

    let template_dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().join("docs/templates");
    let _ = std::fs::create_dir_all(&template_dir);

    for (kind, expected) in &regions_with_labels {
        let region = match cal.regions.get(kind) {
            Some(r) => r,
            None => continue,
        };

        let cropped = crop_region(
            rgba.as_raw(), img_w, img_h,
            region.x, region.y, region.width, region.height,
        );
        let cropped = match cropped {
            Some(c) => c,
            None => {
                eprintln!("  {:?}: crop out of bounds", kind);
                continue;
            }
        };

        let gray = to_grayscale(&cropped);
        let binary = threshold(&gray, 160);

        let boxes = segment_characters(&binary, 2);
        let expected_chars: Vec<char> = expected.chars().collect();

        eprintln!("\n{:?} -> expected \"{}\"", kind, expected);
        eprintln!("  Found {} character boxes (expected {})", boxes.len(), expected_chars.len());

        if boxes.len() != expected_chars.len() {
            eprintln!("  MISMATCH: box count != expected chars, skipping");
            // Save the binary for debugging
            let _ = binary.save(base.join(format!("extract_binary_{:?}.png", kind)));
            for (i, b) in boxes.iter().enumerate() {
                eprintln!("    Box {}: x={}, y={}, w={}, h={}", i, b.x, b.y, b.width, b.height);
            }
            continue;
        }

        for (i, (b, &ch)) in boxes.iter().zip(expected_chars.iter()).enumerate() {
            let sub = image::imageops::crop_imm(&binary, b.x, b.y, b.width, b.height).to_image();
            let filename = format!("template_{}_{}.png", ch.to_string().replace('/', "slash").replace(':', "colon"), i);
            let path = template_dir.join(&filename);
            let _ = sub.save(&path);
            eprintln!("  Saved '{}' -> {:?} ({}x{})", ch, path, b.width, b.height);
        }
    }
}
