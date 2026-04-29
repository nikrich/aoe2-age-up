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
/// Filters out boxes shorter than 40% of the image height (noise).
pub fn segment_characters(binary: &GrayImage, min_width: u32) -> Vec<CharBox> {
    let (w, h) = binary.dimensions();
    if w == 0 || h == 0 {
        return Vec::new();
    }

    let col_has_pixel: Vec<bool> = (0..w)
        .map(|x| (0..h).any(|y| binary.get_pixel(x, y).0[0] > 0))
        .collect();

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

    // Filter out noise: boxes must be at least 40% of image height
    let min_height = (h * 2 / 5).max(3);
    boxes.retain(|b| b.height >= min_height);

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
        let img = make_binary(10, 10, &[2..5]);
        let boxes = segment_characters(&img, 1);
        assert_eq!(boxes.len(), 1);
        assert_eq!(boxes[0].x, 2);
        assert_eq!(boxes[0].width, 3);
    }

    #[test]
    fn test_two_characters_with_gap() {
        let img = make_binary(12, 8, &[1..3, 6..9]);
        let boxes = segment_characters(&img, 1);
        assert_eq!(boxes.len(), 2);
        assert_eq!(boxes[0].x, 1);
        assert_eq!(boxes[1].x, 6);
    }

    #[test]
    fn test_min_width_filters_noise() {
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
