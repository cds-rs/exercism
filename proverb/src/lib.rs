pub fn build_proverb(list: &[&str]) -> String { // [R] list (borrowed slice)
    if list.is_empty() {
        return String::new();
    }

    let mut proverb = Vec::<String>::new(); // [RWO] proverb (owned)
    for pair in list.windows(2) {           // [R] pair (borrowed slice)
        let [want, lost] = pair else { unreachable!() }; // [R] want, lost (borrowed &str)
        proverb.push(format!("For want of a {want} the {lost} was lost."))
    } // borrows end: pair, want, lost

    proverb.push(format!("And all for the want of a {}.", list[0]));
    proverb.join("\n")
} // borrow ends: list; drop: proverb
