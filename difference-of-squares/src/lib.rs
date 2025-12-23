pub fn square_of_sum(n: u32) -> u32 {
    // a: sum of first n = n (n-1) / 2
    // square of sum: a * a
    let sum_1_to_n = n * (n + 1) / 2;
    sum_1_to_n * sum_1_to_n
}

pub fn sum_of_squares(n: u32) -> u32 {
    // sum of squares: n (n+1) (2n + 1) / 6
    n * (n + 1) * (2 * n + 1) / 6
}

pub fn difference(n: u32) -> u32 {
    square_of_sum(n) - sum_of_squares(n)
}
