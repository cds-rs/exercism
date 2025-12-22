use std::collections::HashMap;

pub fn lowest_price(books: &[u32]) -> u32 {
    //                books: &[u32]         -- shared borrow, R only

    // book_counts[i] = number of copies of book (i+1)
    let mut book_counts = [0u32; 5];       // book_counts: O R W
    for &book in books {                   // &book pattern: copies u32 out (Copy)
        book_counts[(book - 1) as usize] += 1;
    }

    let mut cache = HashMap::new();        // cache: O R W
    find_min(book_counts, &mut cache)      // book_counts: copied (Copy), cache: &mut borrow
}                                          // cache: dropped here

fn find_min(counts: [u32; 5], cache: &mut HashMap<[u32; 5], u32>) -> u32 {
    //         counts: [u32; 5]   -- owned (caller's copy), O R W
    //         cache: &mut ...    -- mutable borrow, R W (not O)
    //
    // Q: How can &mut cache exist in every recursive frame?
    // A: Reborrowing: when we pass `cache` to find_min(next, cache):
    //    1. Current frame's &mut is "lent" to callee (suspended)
    //    2. Callee gets a fresh &mut (reborrow) with shorter lifetime
    //    3. Callee returns → caller's &mut reactivates
    //    Only ONE frame actively holds &mut at any instant.
    //    Call stack enforces non-overlapping access.

    // Canonicalize: sort descending to collapse equivalent states
    // e.g., [2,1,1,1,2], [1,2,2,1,1] => [2,2,1,1,1]
    let book_groups = {
        let mut k = counts;                // k: copies counts (Copy), O R W
        k.sort_by(|a, b| b.cmp(a));        // a, b: &u32 refs for comparison
        k                                  // k: moved out of block
    };                                     // book_groups: O R W (owns k)

    if book_groups.iter().all(|&c| c == 0) {  // .iter(): temp & borrow, &c: copies u32
        return 0;
    }

    if let Some(&price) = cache.get(&book_groups) {  // &book_groups: temp & borrow
        return price;                                // &price pattern: copies u32 out
    }

    let max_group = book_groups.iter().filter(|&&c| c > 0).count();  // &&c: deref twice
    let mut min_price = u32::MAX;                                    // min_price: O R W

    // Try each group size; greedy (max only) doesn't yield optimal
    for group_size in 1..=max_group {     // group_size: usize, Copy

        let mut next = book_groups;       // next: copies book_groups (Copy), O R W

        for count in next.iter_mut().take(group_size) {
            //  count: &mut u32            -- exclusive borrow of next[i]
            *count -= 1;                  // deref to mutate
        }                                 // count borrows end here

        let price = group_price(group_size)  // group_size: copied (Copy)
            + find_min(next, cache);         // next: copied (Copy), cache: reborrow &mut
        min_price = min_price.min(price);
    }                                     // next: dropped each iteration

    cache.insert(book_groups, min_price); // book_groups: copied into key (Copy)
    min_price                             // min_price: copied out (Copy)
}

fn group_price(size: usize) -> u32 {
    match size {
        1 => 800,
        2 => 1520, //  5% off
        3 => 2160, // 10% off
        4 => 2560, // 20% off
        5 => 3000, // 25% off
        _ => unreachable!("gropu size must be 1-5, got {size}"),
    }
}
