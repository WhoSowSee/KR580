/// Requires a non-empty ring and a current index inside that ring.
pub(super) fn cycle_index(current: Option<usize>, count: usize, backward: bool) -> usize {
    match (current, backward) {
        (None, false) => 0,
        (None, true) => count - 1,
        (Some(index), false) => (index + 1) % count,
        (Some(index), true) => (index + count - 1) % count,
    }
}
