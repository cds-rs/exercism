/// Check a Luhn checksum.
pub fn is_valid(code: &str) -> bool {
    if code.len() < 2 {
        return false;
    }

    let mut sum = 0;
    let mut i = 0;
    for ch in code.chars().rev() {
        if ch.is_whitespace() {
            continue;
        }

        if !ch.is_ascii_digit() {
            return false;
        }

        let mut dig = ch.to_digit(10).unwrap();
        if i % 2 == 1 {
            dig *= 2;
            if dig > 9 {
                dig -= 9;
            }
        }
        sum += dig;
        i += 1;
    }
    if i > 1 { sum % 10 == 0 } else { false }
}
