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
