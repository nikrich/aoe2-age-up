use image::{Rgba, RgbaImage};
use imageproc::drawing::draw_hollow_rect_mut;
use imageproc::rect::Rect;
use xcap::Monitor;

#[test]
fn save_calibration_preview() {
    // Capture the primary monitor
    let monitors = Monitor::all().expect("Failed to list monitors");
    let monitor = monitors.into_iter().next().expect("No monitor found");
    let capture = monitor.capture_image().expect("Failed to capture screen");

    let mut img: RgbaImage = capture;

    // Current region definitions (must match state.rs)
    let regions: Vec<(&str, u32, u32, u32, u32, Rgba<u8>)> = vec![
        // (name, x, y, width, height, color)
        ("Wood",       50,  20, 48, 20, Rgba([0, 255, 0, 255])),     // Green
        ("Food",       143, 20, 48, 20, Rgba([255, 0, 0, 255])),     // Red
        ("Gold",       243, 20, 48, 20, Rgba([255, 255, 0, 255])),   // Yellow
        ("Stone",      338, 20, 48, 20, Rgba([128, 128, 128, 255])), // Gray
        ("Villagers",  420, 30, 28, 18, Rgba([0, 128, 255, 255])),   // Blue
        ("Population", 450, 20, 55, 20, Rgba([255, 0, 255, 255])),   // Magenta
        ("GameTime",   830, 2,  90, 22, Rgba([255, 128, 0, 255])),   // Orange
    ];

    for (name, x, y, w, h, color) in &regions {
        // Draw 2px thick box by drawing rect and rect-1
        let rect = Rect::at(*x as i32, *y as i32).of_size(*w, *h);
        draw_hollow_rect_mut(&mut img, rect, *color);
        let rect2 = Rect::at(*x as i32 + 1, *y as i32 + 1).of_size(w - 2, h - 2);
        draw_hollow_rect_mut(&mut img, rect2, *color);
        println!("{}: x={}, y={}, w={}, h={}", name, x, y, w, h);
    }

    // Crop to just the top bar area for easier viewing
    let top_bar = image::imageops::crop_imm(&img, 0, 0, img.width().min(960), 50);
    let top_bar_img = top_bar.to_image();
    top_bar_img.save("E:/Development/open-age/calibration_preview.png").expect("Failed to save");

    // Also save full screenshot for reference
    img.save("E:/Development/open-age/calibration_full.png").expect("Failed to save full");

    println!("Saved calibration_preview.png and calibration_full.png");
}
