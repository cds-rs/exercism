// Fold over digits with O(1) space; a function that takes a closure.
// Captures the two-loop pattern without iterator/Vec overhead.
fn fold_digits(mut n: u32, init: u32, mut f: impl FnMut(u32, u32) -> u32) -> u32 {
    let mut acc = init;
    loop {
        acc = f(acc, n % 10);
        n /= 10;
        if n == 0 {
            break acc;
        }
    }
}

pub fn is_armstrong_number(num: u32) -> bool {
    let digit_count = fold_digits(num, 0, |count, _| count + 1);
    let sum_of_powers = fold_digits(num, 0, |sum, digit| sum + digit.pow(digit_count));

    sum_of_powers == num
}
