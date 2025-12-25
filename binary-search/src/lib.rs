pub fn find<T: Ord>(array: impl AsRef<[T]>, key: T) -> Option<usize> {
    let ary = array.as_ref();

    let mut l = 0;
    let mut r = ary.len();

    while l < r {
        let m = (l + r) / 2;
        if ary[m] == key {
            return Some(m);
        }

        if key < ary[m] {
            r = m;
        } else {
            l = m + 1;
        }
    }
    None
}
