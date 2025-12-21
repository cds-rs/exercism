#[cfg(feature = "grapheme")]
use unicode_segmentation::UnicodeSegmentation;

#[cfg(feature = "grapheme")]
pub fn reverse(input: &str) -> String {
    input.graphemes(true).rev().collect()
}

#[cfg(not(feature = "grapheme"))]
pub fn reverse(input: &str) -> String {
    // conditionally compiling to default to not graphemed
    input.chars().rev().collect()
}
