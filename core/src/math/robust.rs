//! Deterministic pseudo-random support for reproducible robust estimators.

/// Tiny deterministic generator used only for reproducible sampling in MP2.
#[derive(Clone, Debug)]
pub struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    /// Creates a generator from a caller-controlled seed.
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }
    /// Returns a uniformly distributed index below `upper`, or zero for an empty range.
    pub fn index(&mut self, upper: usize) -> usize {
        if upper == 0 {
            return 0;
        }
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        ((self.state >> 32) as usize) % upper
    }
}
