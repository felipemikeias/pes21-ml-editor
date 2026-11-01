//! MT19937 (32-bit Mersenne Twister) as published by Matsumoto & Nishimura
//! (`mt19937ar.c`, 2002-01-26). Only the pieces the PES save cipher needs are
//! implemented: seeding with `init_by_array` and drawing 32-bit outputs, plus the
//! inverse pieces [`crate::recover`] needs.

pub(crate) const N: usize = 624;
pub(crate) const M: usize = 397;
pub(crate) const MATRIX_A: u32 = 0x9908_b0df;
const UPPER_MASK: u32 = 0x8000_0000;
const LOWER_MASK: u32 = 0x7fff_ffff;
/// Seed `init_by_array` starts from.
pub(crate) const INIT_BY_ARRAY_SEED: u32 = 19_650_218;
/// Multipliers of the two `init_by_array` loops.
pub(crate) const INIT_MUL_1: u32 = 1_664_525;
pub(crate) const INIT_MUL_2: u32 = 1_566_083_941;

/// Output tempering of `genrand_int32`.
pub(crate) fn temper(mut y: u32) -> u32 {
    y ^= y >> 11;
    y ^= (y << 7) & 0x9d2c_5680;
    y ^= (y << 15) & 0xefc6_0000;
    y ^ (y >> 18)
}

/// Inverse of [`temper`].
pub(crate) fn untemper(mut y: u32) -> u32 {
    y ^= y >> 18;
    y ^= (y << 15) & 0xefc6_0000;
    // Each pass recovers 7 (then 11) more bits of the shifted-in value.
    let shl7 = y;
    for _ in 0..4 {
        y = shl7 ^ ((y << 7) & 0x9d2c_5680);
    }
    let shr11 = y;
    for _ in 0..2 {
        y = shr11 ^ (y >> 11);
    }
    y
}

/// State of `init_genrand(seed)`.
pub(crate) fn seeded_state(seed: u32) -> [u32; N] {
    let mut mt = [0u32; N];
    mt[0] = seed;
    for i in 1..N {
        mt[i] = 1_812_433_253u32
            .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
            .wrapping_add(i as u32);
    }
    mt
}

/// Mersenne Twister state.
#[derive(Clone)]
pub struct Mt19937 {
    mt: [u32; N],
    index: usize,
}

impl Mt19937 {
    /// Equivalent of `init_genrand(s)`.
    fn with_seed(seed: u32) -> Self {
        Self {
            mt: seeded_state(seed),
            index: N,
        }
    }

    /// Equivalent of `init_by_array(key, key.len())`.
    ///
    /// # Panics
    /// Panics if `key` is empty (the reference implementation would read out of bounds).
    pub fn from_key(key: &[u32]) -> Self {
        assert!(!key.is_empty(), "MT19937 key must not be empty");
        let mut s = Self::with_seed(INIT_BY_ARRAY_SEED);
        let mt = &mut s.mt;
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..N.max(key.len()) {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(INIT_MUL_1))
                .wrapping_add(key[j])
                .wrapping_add(j as u32);
            i += 1;
            j += 1;
            if i >= N {
                mt[0] = mt[N - 1];
                i = 1;
            }
            if j >= key.len() {
                j = 0;
            }
        }
        for _ in 0..N - 1 {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(INIT_MUL_2))
                .wrapping_sub(i as u32);
            i += 1;
            if i >= N {
                mt[0] = mt[N - 1];
                i = 1;
            }
        }
        mt[0] = 0x8000_0000;
        s
    }

    fn twist(&mut self) {
        for k in 0..N {
            let y = (self.mt[k] & UPPER_MASK) | (self.mt[(k + 1) % N] & LOWER_MASK);
            let mag = if y & 1 == 1 { MATRIX_A } else { 0 };
            self.mt[k] = self.mt[(k + M) % N] ^ (y >> 1) ^ mag;
        }
        self.index = 0;
    }

    /// Equivalent of `genrand_int32()`.
    pub fn next_u32(&mut self) -> u32 {
        if self.index >= N {
            self.twist();
        }
        let y = self.mt[self.index];
        self.index += 1;
        temper(y)
    }

    /// The 624 words `init_by_array` leaves behind, before the first twist.
    #[cfg(test)]
    pub(crate) fn initial_state(&self) -> [u32; N] {
        self.mt
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// First outputs of the reference `mt19937ar.c` test program
    /// (`init_by_array({0x123, 0x234, 0x345, 0x456}, 4)`).
    #[test]
    fn matches_reference_output() {
        let mut mt = Mt19937::from_key(&[0x123, 0x234, 0x345, 0x456]);
        let expected = [
            1_067_595_299u32,
            955_945_823,
            477_289_528,
            4_107_218_783,
            4_228_976_476,
        ];
        for e in expected {
            assert_eq!(mt.next_u32(), e);
        }
        // Output #1000 of the reference listing (index 999).
        let mut mt = Mt19937::from_key(&[0x123, 0x234, 0x345, 0x456]);
        let mut last = 0;
        for _ in 0..1000 {
            last = mt.next_u32();
        }
        assert_eq!(last, 3_460_025_646);
    }

    #[test]
    fn untemper_inverts_temper() {
        for x in [0, 1, 0x8000_0000, 0xffff_ffff, 0x1234_5678, 0x9908_b0df] {
            assert_eq!(untemper(temper(x)), x);
        }
        let mut x = 0x2545_f491u32;
        for _ in 0..10_000 {
            x = x.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
            assert_eq!(untemper(temper(x)), x);
        }
    }
}
