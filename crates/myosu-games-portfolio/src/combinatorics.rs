/// Produce at most `limit` unordered pairs from a slice.
pub fn bounded_pairs<T: Copy>(items: &[T], limit: usize) -> Vec<(T, T)> {
    let mut pairs = Vec::new();
    for (left_index, left) in items.iter().copied().enumerate() {
        for right in items.iter().copied().skip(left_index.saturating_add(1)) {
            if pairs.len() >= limit {
                return pairs;
            }
            pairs.push((left, right));
        }
    }

    pairs
}

/// Produce at most `limit` unordered triples from a slice.
pub fn bounded_triples<T: Copy>(items: &[T], limit: usize) -> Vec<[T; 3]> {
    let mut triples = Vec::new();
    for (first_index, first) in items.iter().copied().enumerate() {
        for (second_offset, second) in items
            .iter()
            .copied()
            .skip(first_index.saturating_add(1))
            .enumerate()
        {
            let third_start = first_index
                .saturating_add(1)
                .saturating_add(second_offset)
                .saturating_add(1);
            for third in items.iter().copied().skip(third_start) {
                if triples.len() >= limit {
                    return triples;
                }
                triples.push([first, second, third]);
            }
        }
    }

    triples
}

#[cfg(test)]
mod tests {
    use crate::combinatorics::{bounded_pairs, bounded_triples};

    #[test]
    fn bounded_pairs_are_deterministic_and_limited() {
        let pairs = bounded_pairs(&[1, 2, 3, 4], 3);

        assert_eq!(pairs, vec![(1, 2), (1, 3), (1, 4)]);
    }

    #[test]
    fn bounded_triples_are_deterministic_and_limited() {
        let triples = bounded_triples(&[1, 2, 3, 4], 2);

        assert_eq!(triples, vec![[1, 2, 3], [1, 2, 4]]);
    }

    #[test]
    fn zero_budget_returns_no_combinations() {
        assert!(bounded_pairs(&[1, 2, 3], 0).is_empty());
        assert!(bounded_triples(&[1, 2, 3], 0).is_empty());
    }
}
