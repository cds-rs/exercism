fn fold_digits(mut n: u32, init: u32, f: impl Fn(u32, u32) -> u32) -> u32 {
    let mut acc = init;
    while n > 0 {
        acc = f(acc, n % 10);
        n /= 10;
    }
    acc
}

pub fn is_armstrong_number(num: u32) -> bool {
    // TIL: iLog10! thanks Timus
    let digit_count = if num == 0 { 1 } else { num.ilog10() + 1 };
    let sum_of_powers = fold_digits(num, 0, |sum, digit| sum + digit.pow(digit_count));
    sum_of_powers == num
}

