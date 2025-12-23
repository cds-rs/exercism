pub fn is_armstrong_number(num: u32) -> bool {
    let mut remaining = num;
    let mut sum_of_powers = 0;
    let mut digit_count = 1;

    // Constraint: avoid format! to count digits.

    // Pattern: iterate over digits, accumulate a count.
    // This is equivalent to: digits.len()
    loop {
        remaining /= 10;
        if remaining == 0 {
            break;
        }
        digit_count += 1;
    }

    // Pattern: iterate over digits, transform each, accumulate sum.
    // This is equivalent to: digits.map(|d| d.pow(n)).sum()
    remaining = num;
    loop {
        let digit = remaining % 10;
        sum_of_powers += digit.pow(digit_count);
        remaining /= 10;
        if remaining == 0 {
            break;
        }
    }

    sum_of_powers == num
}
