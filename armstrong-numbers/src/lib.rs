pub fn is_armstrong_number(num: u32) -> bool {
    let mut remaining = num;
    let mut sum_of_powers = 0;
    let digit_count = format!("{num}").len() as u32;

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
