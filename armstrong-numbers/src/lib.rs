// Timus soln, amazing!
pub fn is_armstrong_number(num: u32) -> bool {
    if num < 10 { return true }

    let digit_count = num.ilog10() + 1;
    let mut sum_of_powers = 0;
    let mut n = num;
    while n > 0 {
        sum_of_powers += (n % 10).pow(digit_count);
        n /= 10;
    }

    sum_of_powers == num
}
