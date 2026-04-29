// Tesseract OCR backend (feature-gated behind `tesseract` feature)
// TODO: Implement using `rusty-tesseract` crate with:
// - tessedit_char_whitelist = "0123456789/:"
// - psm = 7 (single text line)
// - oem = 3 (default LSTM engine)
// See spec.md §5.3 for implementation details.
