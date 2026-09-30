/// End-to-end test of the Tesseract OCR pipeline the app actually uses at
/// runtime: same binary/tessdata resolution as `ipc::start_capture`, same
/// preprocessing (grayscale -> threshold -> upscale), same calibrated regions.
///
/// Source image: docs/aoe.png (AoE2:DE at 1919x1079) with known values:
/// Wood=200, Food=200, Gold=100, Stone=200, Villagers=3, Pop=4/5, Time=00:00:05.
use aoe_overlay::ipc::resolve_tesseract;
use aoe_overlay::ocr::preprocess::crop_region;
use aoe_overlay::ocr::windows_ocr::TesseractPipeline;
use aoe_overlay::ocr::OcrPipeline;
use aoe_overlay::state::{Calibration, RegionKind};

fn screenshot() -> Option<image::RgbaImage> {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join("docs/aoe.png");
    if !path.exists() {
        eprintln!("screenshot not found, skipping: {:?}", path);
        return None;
    }
    Some(image::open(&path).expect("failed to open screenshot").to_rgba8())
}

fn pipeline() -> Option<TesseractPipeline> {
    let exe_dir = std::env::current_exe().ok()?.parent()?.to_path_buf();
    let (tesseract, deps, tessdata) = resolve_tesseract(
        &exe_dir,
        &std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR")),
    );
    eprintln!("tesseract={:?} tessdata={:?}", tesseract, tessdata);
    match TesseractPipeline::new(tesseract, tessdata, deps) {
        Ok(p) => Some(p),
        Err(e) => {
            eprintln!("tesseract unavailable, skipping: {e:#}");
            None
        }
    }
}

fn read_number(
    p: &TesseractPipeline,
    img: &image::RgbaImage,
    kind: RegionKind,
) -> Option<u32> {
    let cal = Calibration::default_1080p();
    let r = cal.regions.get(&kind)?;
    let crop = crop_region(
        img.as_raw(),
        img.width(),
        img.height(),
        r.x,
        r.y,
        r.width,
        r.height,
    )?;
    let (w, h) = (crop.width(), crop.height());
    p.read_number(crop.as_raw(), w, h, kind).ok().flatten()
}

#[test]
fn tesseract_reads_resource_numbers_from_real_screenshot() {
    let (Some(img), Some(p)) = (screenshot(), pipeline()) else {
        return;
    };

    for (kind, expected) in [
        (RegionKind::Wood, 200),
        (RegionKind::Food, 200),
        (RegionKind::Gold, 100),
        (RegionKind::Stone, 200),
        (RegionKind::Villagers, 3),
    ] {
        let got = read_number(&p, &img, kind);
        assert_eq!(got, Some(expected), "OCR of {:?} mismatched", kind);
    }
}

#[test]
fn tesseract_reads_population_from_real_screenshot() {
    let (Some(img), Some(p)) = (screenshot(), pipeline()) else {
        return;
    };
    let cal = Calibration::default_1080p();
    let r = cal.regions.get(&RegionKind::Population).unwrap();
    let crop = crop_region(
        img.as_raw(),
        img.width(),
        img.height(),
        r.x,
        r.y,
        r.width,
        r.height,
    )
    .unwrap();
    let (w, h) = (crop.width(), crop.height());
    let pop = p.read_population(crop.as_raw(), w, h).ok().flatten();
    assert_eq!(pop, Some((4, 5)), "population OCR mismatched");
}

#[test]
fn tesseract_reads_game_time_from_real_screenshot() {
    let (Some(img), Some(p)) = (screenshot(), pipeline()) else {
        return;
    };
    let cal = Calibration::default_1080p();
    let r = cal.regions.get(&RegionKind::GameTime).unwrap();
    let crop = crop_region(
        img.as_raw(),
        img.width(),
        img.height(),
        r.x,
        r.y,
        r.width,
        r.height,
    )
    .unwrap();
    let (w, h) = (crop.width(), crop.height());
    let _ = init_tracing();
    let t = p.read_time(crop.as_raw(), w, h).ok().flatten();
    eprintln!("game time = {:?}", t);
    assert_eq!(t, Some(5), "game time OCR mismatched");
}

/// Surface the OCR pipeline's `debug!` logs (raw tesseract text) when running
/// with `--nocapture`.
fn init_tracing() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        let _ = tracing_subscriber::fmt()
            .with_max_level(tracing::Level::DEBUG)
            .try_init();
    });
}
