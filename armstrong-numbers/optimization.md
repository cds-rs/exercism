# Armstrong Numbers: Optimization Journey

An exploration of abstraction costs in Rust, from high-level iterators to zero-cost closures.

## The Problem

Check if a number equals the sum of its digits each raised to the power of the digit count.

```rust
// 153 = 1^3 + 5^3 + 3^3 = 1 + 125 + 27 = 153 ✓
```

## Approaches Compared

### 1. Format String (baseline)

```rust
pub fn is_armstrong_number(num: u32) -> bool {
    let digit_count = format!("{num}").len() as u32;
    // ... loop to compute sum
}
```

- Simple, readable
- Allocates a String just to count digits

### 2. Iterator with `successors`

```rust
pub fn is_armstrong_number(num: u32) -> bool {
    let digits: Vec<_> = std::iter::successors(Some(num), |&n| (n >= 10).then_some(n / 10))
        .map(|n| n % 10)
        .collect();

    let digit_count = digits.len() as u32;
    let sum_of_powers: u32 = digits.iter().map(|&d| d.pow(digit_count)).sum();

    sum_of_powers == num
}
```

- Elegant, functional style
- Collects into a Vec (heap allocation)

### 3. Double Loop (no allocation)

```rust
pub fn is_armstrong_number(num: u32) -> bool {
    let mut remaining = num;
    let mut digit_count = 1;
    loop {
        remaining /= 10;
        if remaining == 0 { break; }
        digit_count += 1;
    }

    remaining = num;
    let mut sum_of_powers = 0;
    loop {
        sum_of_powers += (remaining % 10).pow(digit_count);
        remaining /= 10;
        if remaining == 0 { break; }
    }

    sum_of_powers == num
}
```

- O(1) space
- Manual, repetitive structure

### 4. Closure Abstraction (best of both)

```rust
fn fold_digits(mut n: u32, init: u32, mut f: impl FnMut(u32, u32) -> u32) -> u32 {
    let mut acc = init;
    loop {
        acc = f(acc, n % 10);
        n /= 10;
        if n == 0 { break acc; }
    }
}

pub fn is_armstrong_number(num: u32) -> bool {
    let digit_count = fold_digits(num, 0, |count, _| count + 1);
    let sum_of_powers = fold_digits(num, 0, |sum, digit| sum + digit.pow(digit_count));
    sum_of_powers == num
}
```

- O(1) space
- Clean abstraction: "a function that takes a closure"
- Captures the fold pattern without iterator overhead

## Big-O Analysis

| Approach | Time | Space |
|----------|------|-------|
| format! + loop | O(d) | O(d): String |
| Iterator + collect | O(d) | O(d): Vec |
| Double loop | O(d) | O(1) |
| Closure fold | O(d) | O(1) |

Where d = number of digits ≈ log₁₀(n)

**But Big-O hides constant factors. What does the compiler actually emit?**

## Assembly Comparison (Godbolt)

Compiled with `-C opt-level=3 -C lto=thin`

### Double Loop vs Closure: Identical!

Both compile to ~50 lines of pure register arithmetic. The closure abstraction compiles away completely; both use the multiplication trick for division (`imul rcx, 3435973837; shr rsi, 35`).

### Iterator Version: Cannot Escape Allocation

~150 lines of assembly. Even with aggressive optimization, the iterator version includes:
- `__rust_alloc`: heap allocation
- `__rust_realloc`: potential reallocation
- `__rust_dealloc`: cleanup
- Exception unwinding machinery

## Summary Table

| Version | ASM Size | Allocations | Function Calls |
|---------|----------|-------------|----------------|
| Double loop | ~50 lines | 0 | 0 |
| Closure fold | ~50 lines | 0 | 0 |
| Iterator | ~150 lines | Yes | alloc, dealloc, reserve |
| format! | ~1000 lines | Yes | fmt machinery |

## Key Takeaways

1. **Zero-cost abstractions are real**: the closure version compiles to identical assembly as hand-written loops

2. **Allocation is the real cost**: iterator's `collect()` forces a Vec that the optimizer cannot eliminate

3. **Big-O lies**: O(d) vs O(d) hides a 3x difference in code size and heap allocations

4. **Design for your constraints**: if you need intermediate storage, use iterators; if not, fold-style abstractions give you clean code with no overhead

5. **Measure with Godbolt**: theoretical analysis only goes so far; check the actual assembly with `-C opt-level=3 -C lto=thin`

## The Pattern

When you see repeated loop structure:

```rust
// Pattern: iterate, accumulate
loop {
    acc = f(acc, item);
    if done { break acc; }
}
```

Extract it as a higher-order function:

```rust
fn fold_over<T>(mut state: S, init: T, mut f: impl FnMut(T, Item) -> T) -> T
```

You get abstraction at the source level with zero runtime cost.

---

## Appendix: Godbolt Links

All examples compiled with `-C opt-level=3 -C lto=thin`

| Version | Link |
|---------|------|
| Double Loop | https://godbolt.org/z/eYb89Khd1 |
| Closure | https://godbolt.org/z/zP544Y61f |
| Iterator | https://godbolt.org/z/9q638Exsz |
