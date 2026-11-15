//! Ports: abstractions the domain depends on and outer layers implement.

/// Source of randomness. The domain never picks a concrete generator.
pub trait RandomSource {
    /// Uniform value in `[0, 1)`.
    fn next_f32(&mut self) -> f32;

    /// Uniform value in `[lo, hi)`.
    fn range(&mut self, lo: f32, hi: f32) -> f32 {
        lo + (hi - lo) * self.next_f32()
    }

    /// `true` with probability `p`.
    fn chance(&mut self, p: f32) -> bool {
        self.next_f32() < p
    }

    /// Uniform index in `0..len` (`0` when `len` is zero).
    fn index(&mut self, len: usize) -> usize {
        if len == 0 {
            return 0;
        }
        ((self.next_f32() * len as f32) as usize).min(len - 1)
    }
}

#[cfg(test)]
pub(crate) mod testing {
    use super::RandomSource;

    /// Small deterministic generator for unit tests.
    pub struct TestRng(pub u64);

    impl RandomSource for TestRng {
        fn next_f32(&mut self) -> f32 {
            self.0 ^= self.0 << 13;
            self.0 ^= self.0 >> 7;
            self.0 ^= self.0 << 17;
            (self.0 >> 40) as f32 / (1u64 << 24) as f32
        }
    }

    /// Always returns the same value.
    pub struct Fixed(pub f32);

    impl RandomSource for Fixed {
        fn next_f32(&mut self) -> f32 {
            self.0
        }
    }
}
