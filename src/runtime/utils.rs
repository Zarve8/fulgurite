pub fn compare_arrays<T: Eq>(a: &[T], b: &[T]) -> bool {
    if a.len() != b.len() {
        return false;
    }

    let unmatched = a.iter().zip(b.iter())
        .filter(|(aa, bb)| *a != *b)
        .count();

    unmatched == 0
}
