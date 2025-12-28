# Armstrong Numbers: Optimization Journey

An exploration of abstraction costs in Rust, from high-level iterators to zero-cost closures and branchless arithmetic.

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

### 4. Closure Abstraction

```rust
fn fold_digits(mut n: u32, init: u32, f: impl Fn(u32, u32) -> u32) -> u32 {
    let mut acc = init;
    while n > 0 {
        acc = f(acc, n % 10);
        n /= 10;
    }
    acc
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

### 5. ilog10 + Closure

```rust
fn fold_digits(mut n: u32, init: u32, f: impl Fn(u32, u32) -> u32) -> u32 {
    let mut acc = init;
    while n > 0 {
        acc = f(acc, n % 10);
        n /= 10;
    }
    acc
}

pub fn is_armstrong_number(num: u32) -> bool {
    let digit_count = if num == 0 { 1 } else { num.ilog10() + 1 };
    let sum_of_powers = fold_digits(num, 0, |sum, digit| sum + digit.pow(digit_count));
    sum_of_powers == num
}
```

- Uses `ilog10()` for O(1) digit counting via branchless arithmetic
- Single pass through digits (only for sum calculation)
- Handles edge case: `0.ilog10()` panics, so special-case it

### 6. Early Return + Inline Loop (current solution)

```rust
pub fn is_armstrong_number(num: u32) -> bool {
    // clever observation: always true for single digits (d^1 = d)
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
```

- Early return for 0-9: all single-digit numbers are Armstrong (d¹ = d)
- No special case needed for `ilog10(0)` since `num < 10` catches it
- Drops the closure abstraction for maximum simplicity
- Single function, no dependencies

## Big-O Analysis

| Approach | Time | Space |
|----------|------|-------|
| format! + loop | O(d) | O(d): String |
| Iterator + collect | O(d) | O(d): Vec |
| Double loop | O(d) | O(1) |
| Closure fold | O(d) | O(1) |
| ilog10 + Closure | O(d) | O(1) |
| Early return + inline | O(d) | O(1) |

Where d = number of digits ≈ log₁₀(n)

Note: `ilog10()` is O(1) via branchless arithmetic with magic constants, reducing digit counting from O(d) to O(1), but overall complexity remains O(d) due to the sum calculation.

**But Big-O hides constant factors. What does the compiler actually emit?**

## Assembly Comparison (Godbolt)

Compiled with `-C opt-level=3 -C lto=thin`

### Double Loop vs Closure: Identical!

Both compile to ~50 lines of pure register arithmetic. The closure abstraction compiles away completely; both use the multiplication trick for division (`imul rcx, 3435973837; shr rsi, 35`).

### ilog10 Version: Even Smaller

The `ilog10` version compiles to ~40 lines. Instead of a loop, it uses branchless arithmetic with magic constants: a `shr`/`imul`/`shr` sequence approximates log2, then bit manipulation with precomputed constants (393206, 524188, etc.) refines the result to exact log10. One comparison handles the 100000 boundary. No loops for digit counting.

### Early Return vs Zero Check

**Approach #5** (ilog10 + Closure) handles zero specially:
```asm
test    edi, edi
je      .LBB0_1         ; jump if num == 0
```
Still runs full ilog10 arithmetic for 1-9.

**Approach #6** (Early Return) catches all single digits:
```asm
cmp     edi, 10
jb      .LBB0_8         ; jump if num < 10
```
Skips ilog10 entirely for 0-9. For small inputs, this avoids ~15 instructions of magic constant arithmetic.

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
| ilog10 + Closure | ~60 lines | 0 | 0 |
| Early return + inline | ~57 lines | 0 | 0 |
| Iterator | ~150 lines | Yes | alloc, dealloc, reserve |
| format! | ~1000 lines | Yes | fmt machinery |

## Key Takeaways

1. **Zero-cost abstractions are real**: the closure version compiles to identical assembly as hand-written loops

2. **Allocation is the real cost**: iterator's `collect()` forces a Vec that the optimizer cannot eliminate

3. **Big-O lies**: O(d) vs O(d) hides a 3x difference in code size and heap allocations

4. **Design for your constraints**: if you need intermediate storage, use iterators; if not, fold-style abstractions give you clean code with no overhead

5. **Branchless beats loops**: `ilog10()` uses arithmetic tricks with magic constants for O(1) digit counting, eliminating one full pass through the number

6. **Early returns simplify edge cases**: returning early for single digits avoids `ilog10(0)` panic without conditional logic in the main path

7. **Measure with Godbolt**: theoretical analysis only goes so far; check the actual assembly with `-C opt-level=3 -C lto=thin`

8. Most importantly, know your libraries. `iLog10()` is a good technique for approximating number of digits, and LLVM optimizes it.

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
| iLog | https://godbolt.org/z/GaqT8z16x |
| Early return + inline | https://godbolt.org/z/3dvPrxsnq |
