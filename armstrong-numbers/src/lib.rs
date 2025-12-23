pub fn is_armstrong_number(num: u32) -> bool {

    // Extract "iterate over digits" into an iterator.
    // The loop patterns from before become: .len() and .map(f).sum()

    // then(|| v) defers computing v until the condition is true (lazy).
    // then_some(v) computes v first, then wraps in Some if true (eager).
    // Prefer then_some when v is trivial; laziness adds no benefit here.
    let digits: Vec<_> = std::iter::successors(Some(num), |&n| (n >= 10).then_some(n / 10))
        .map(|n| n % 10)
        .collect();

    let digit_count = digits.len() as u32;
    let sum_of_powers: u32 = digits.iter().map(|&d| d.pow(digit_count)).sum();

    sum_of_powers == num
}
