/// Diagnostic: crop specific areas to precisely locate resource numbers.
#[test]
fn find_resource_bar_positions() {
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
    let (img_w, _img_h) = (rgba.width(), rgba.height());

    let base = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent().unwrap().join("docs");

    // Full top bar - wide view
    let top = image::imageops::crop_imm(&rgba, 0, 0, img_w.min(650), 35);
    let _ = top.to_image().save(base.join("crop_top_bar.png"));

    // Crop 80px wide chunks across the top bar to find each number
    for start_x in (0..600).step_by(40) {
        let chunk = image::imageops::crop_imm(&rgba, start_x, 5, 80, 30);
        let _ = chunk.to_image().save(base.join(format!("chunk_x{}.png", start_x)));
    }

    eprintln!("Saved chunks from x=0 to x=600 in 40px steps (80px wide each)");
    eprintln!("Look at docs/chunk_x*.png to find where each number starts");
}
