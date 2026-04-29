use image::GrayImage;
use std::collections::HashMap;

/// Generate simple synthetic digit templates for testing.
/// Each template is a 10x14 binary image with a crude digit pattern.
pub fn generate_fixture_templates() -> HashMap<char, GrayImage> {
    let mut templates = HashMap::new();
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
