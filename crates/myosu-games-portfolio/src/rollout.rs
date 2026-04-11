/// Small deterministic SplitMix64 sampler for rule-engine rollouts.
#[derive(Clone, Debug)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    pub fn next_index(&mut self, upper_bound: usize) -> Option<usize> {
        if upper_bound == 0 {
            return None;
        }

        let upper_bound = u64::try_from(upper_bound).ok()?;
        let value = self.next_u64().checked_rem(upper_bound)?;
        usize::try_from(value).ok()
    }

    pub fn sample_one<T: Copy>(&mut self, items: &[T]) -> Option<T> {
        let index = self.next_index(items.len())?;
        items.get(index).copied()
    }
}

pub fn sample_with_budget<T: Copy>(items: &[T], budget: usize, seed: u64) -> Vec<T> {
    let mut rng = DeterministicRng::new(seed);
    std::iter::repeat_with(|| rng.sample_one(items))
        .take(budget)
        .flatten()
        .collect()
}

#[cfg(test)]
mod tests {
    use crate::rollout::{DeterministicRng, sample_with_budget};

    #[test]
    fn sampler_is_deterministic_for_same_seed() {
        let left = sample_with_budget(&[10, 20, 30], 8, 99);
        let right = sample_with_budget(&[10, 20, 30], 8, 99);
        let different = sample_with_budget(&[10, 20, 30], 8, 100);

        assert_eq!(left, right);
        assert_ne!(left, different);
    }

    #[test]
    fn empty_items_produce_no_samples() {
        assert!(sample_with_budget::<u8>(&[], 8, 99).is_empty());
    }

    #[test]
    fn zero_upper_bound_has_no_index() {
        let mut rng = DeterministicRng::new(1);

        assert_eq!(rng.next_index(0), None);
    }
}
