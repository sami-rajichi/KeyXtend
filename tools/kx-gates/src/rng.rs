//! Tiny seeded xorshift64* generator: the same seed gives the same numbers.
#![cfg(windows)]

/// Output multiplier of xorshift64*.
const MUL: u64 = 0x2545_F491_4F6C_DD1D;
/// Golden-ratio constant that spreads small seeds over the state.
const SPREAD: u64 = 0x9E37_79B9_7F4A_7C15;
/// Shifts of the xorshift step.
const SHIFTS: (u32, u32, u32) = (12, 25, 27);

/// A seeded random source.
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    /// A generator for `seed`; the state is never zero.
    pub fn new(seed: u64) -> Self {
        Self(seed.wrapping_mul(SPREAD) | 1)
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x >> SHIFTS.0;
        x ^= x << SHIFTS.1;
        x ^= x >> SHIFTS.2;
        self.0 = x;
        x.wrapping_mul(MUL)
    }

    /// A number in `0..n`; 0 when `n` is 0.
    pub fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            return 0;
        }
        let n = u64::try_from(n).unwrap_or(u64::MAX);
        usize::try_from(self.next_u64() % n).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_seed_same_numbers() {
        let (mut a, mut b) = (Rng::new(42), Rng::new(42));
        let xs: Vec<u64> = (0..100).map(|_| a.next_u64()).collect();
        let ys: Vec<u64> = (0..100).map(|_| b.next_u64()).collect();
        assert_eq!(xs, ys);
    }

    #[test]
    fn different_seeds_differ_and_zero_works() {
        let (mut a, mut b) = (Rng::new(0), Rng::new(1));
        assert_ne!(a.next_u64(), b.next_u64());
        assert_ne!(Rng::new(0).next_u64(), 0);
    }

    #[test]
    fn below_stays_in_range() {
        let mut r = Rng::new(7);
        assert!((0..1000).all(|_| r.below(13) < 13));
        assert_eq!(r.below(0), 0);
    }
}
