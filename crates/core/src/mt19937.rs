//! MT19937 (32-bit Mersenne Twister) as published by Matsumoto & Nishimura
//! (`mt19937ar.c`, 2002-01-26). Only the pieces the PES save cipher needs are
//! implemented: seeding with `init_by_array` and drawing 32-bit outputs.

const N: usize = 624;
const M: usize = 397;
const MATRIX_A: u32 = 0x9908_b0df;
const UPPER_MASK: u32 = 0x8000_0000;
const LOWER_MASK: u32 = 0x7fff_ffff;

/// Mersenne Twister state.
#[derive(Clone)]
pub struct Mt19937 {
    mt: [u32; N],
    index: usize,
}

impl Mt19937 {
    /// Equivalent of `init_genrand(s)`.
    fn with_seed(seed: u32) -> Self {
        let mut mt = [0u32; N];
        mt[0] = seed;
        for i in 1..N {
            mt[i] = 1_812_433_253u32
                .wrapping_mul(mt[i - 1] ^ (mt[i - 1] >> 30))
                .wrapping_add(i as u32);
        }
        Self { mt, index: N }
    }

    /// Equivalent of `init_by_array(key, key.len())`.
    ///
    /// # Panics
    /// Panics if `key` is empty (the reference implementation would read out of bounds).
    pub fn from_key(key: &[u32]) -> Self {
        assert!(!key.is_empty(), "MT19937 key must not be empty");
        let mut s = Self::with_seed(19_650_218);
        let mt = &mut s.mt;
        let (mut i, mut j) = (1usize, 0usize);
        for _ in 0..N.max(key.len()) {
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1_664_525))
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
            mt[i] = (mt[i] ^ (mt[i - 1] ^ (mt[i - 1] >> 30)).wrapping_mul(1_566_083_941))
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
        let mut y = self.mt[self.index];
        self.index += 1;
        y ^= y >> 11;
        y ^= (y << 7) & 0x9d2c_5680;
        y ^= (y << 15) & 0xefc6_0000;
        y ^= y >> 18;
        y
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
}
