pub fn series(digits: &str, len: usize) -> Vec<String> {
    if len == 0 || len > digits.len() { return vec![]; }

    digits 
        .as_bytes()
        .windows(len)
        .map(|w| str::from_utf8(w).unwrap().to_string())
        .collect()
}
