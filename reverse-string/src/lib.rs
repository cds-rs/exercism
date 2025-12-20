use unicode_segmentation::UnicodeSegmentation;
pub fn reverse(input: &str) -> String {
    // why is this failing to compile on exercism?
    input.graphemes(true).rev().collect()
}
