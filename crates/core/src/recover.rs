//! Recovers a save's keys from the file alone, for games whose master key is not known.
//!
//! The block cipher is a keystream XOR (see [`crate::crypto::crypt_in_place`]): each
//! keystream word is the XOR of rotations of five consecutive MT19937 outputs, and
//! MT19937 is linear over GF(2) except for its seeding. A stretch of about 2.6 KiB whose
//! plaintext is known therefore gives the generator away. Master League data keeps large
//! zero-filled areas, which serve as that known plaintext:
//!
//! 1. Guess that a window of the file holds a constant plaintext word (all zeros or all
//!    ones). The keystream there is then known. Writing the four generator outputs that
//!    precede the window as 128 unknown bits, every later output is an affine function
//!    of them.
//! 2. The MT19937 recurrence `y[n+624] = y[n+397] ^ twist(y[n], y[n+1])` (on untempered
//!    outputs) gives linear equations in those bits. Gaussian elimination either solves
//!    them or proves the guess wrong.
//! 3. Running the recurrence backwards reaches the state `init_by_array` left behind,
//!    which is inverted to read the 16-word block key. Where the block starts is not
//!    known, so every candidate start is tried: `init_by_array` feeds its key in 39
//!    times over, and the copies only agree for the real start.
//! 4. The window is assumed to lie in `data.dat`, whose block key is the rolling key XOR
//!    `2`. The caller verifies the result by decrypting the file header with it.

use crate::mt19937::{
    seeded_state, untemper, INIT_BY_ARRAY_SEED, INIT_MUL_1, INIT_MUL_2, M, MATRIX_A, N,
};

/// Plaintext words assumed for a window: zero-filled areas first, then 0xFF-filled ones.
const PATTERNS: [u32; 2] = [0, u32::MAX];
/// Keystream words per window. Each word past the first `N - 4` adds 32 equations on the
/// 128 unknowns, so this gives 640 equations.
const WINDOW: usize = N + 16;
/// Distance between the windows tried, in bytes. A constant area is found if it is at
/// least `4 × WINDOW + STRIDE` (about 4.6 KiB) long.
const STRIDE: usize = 2048;
/// Block number of `data.dat` in the block key.
const DATA_BLOCK: u64 = 2;
/// Words in a block key.
const KEY_WORDS: usize = 16;
/// Solved windows to examine before giving up. A solved window almost always lies in a
/// constant area of `data.dat`, and every such window leads to the same key, so more
/// would only repeat the (comparatively slow) search for the block start.
const MAX_SOLVED_WINDOWS: usize = 4;

/// A 32-bit word whose bits are affine functions of 128 unknown bits.
#[derive(Clone, Copy)]
struct Affine {
    /// `bits[b]`: which unknowns bit `b` depends on.
    bits: [u128; 32],
    constant: u32,
}

impl Affine {
    fn constant(constant: u32) -> Self {
        Self {
            bits: [0; 32],
            constant,
        }
    }

    /// Unknown word `index` (0–3): bit `b` is unknown number `32 × index + b`.
    fn unknown(index: usize) -> Self {
        let mut bits = [0u128; 32];
        for (b, mask) in bits.iter_mut().enumerate() {
            *mask = 1u128 << (32 * index + b);
        }
        Self { bits, constant: 0 }
    }

    fn xor(mut self, other: &Self) -> Self {
        for (a, b) in self.bits.iter_mut().zip(&other.bits) {
            *a ^= b;
        }
        self.constant ^= other.constant;
        self
    }

    fn rotate_right(&self, n: u32) -> Self {
        let mut bits = [0u128; 32];
        for (b, mask) in bits.iter_mut().enumerate() {
            *mask = self.bits[(b + n as usize) % 32];
        }
        Self {
            bits,
            constant: self.constant.rotate_right(n),
        }
    }

    fn rotate_left(&self, n: u32) -> Self {
        self.rotate_right(32 - n)
    }

    /// Applies a GF(2)-linear map given by the images of the 32 unit vectors.
    fn map(&self, images: &[u32; 32], f: fn(u32) -> u32) -> Self {
        let mut bits = [0u128; 32];
        for (i, image) in images.iter().enumerate() {
            for (b, mask) in bits.iter_mut().enumerate() {
                if image >> b & 1 == 1 {
                    *mask ^= self.bits[i];
                }
            }
        }
        Self {
            bits,
            constant: f(self.constant),
        }
    }

    fn eval(&self, x: u128) -> u32 {
        let mut v = self.constant;
        for (b, mask) in self.bits.iter().enumerate() {
            if (mask & x).count_ones() & 1 == 1 {
                v ^= 1 << b;
            }
        }
        v
    }
}

/// MT19937's twist of two untempered words (linear).
fn twist(a: u32, b: u32) -> u32 {
    let y = (a & 0x8000_0000) | (b & 0x7fff_ffff);
    (y >> 1) ^ if y & 1 == 1 { MATRIX_A } else { 0 }
}

/// Incremental Gaussian elimination over GF(2) with 128 unknowns.
struct Equations {
    /// Row whose highest set bit is the index, with its right-hand side.
    pivots: [Option<(u128, bool)>; 128],
}

impl Equations {
    fn new() -> Self {
        Self {
            pivots: [None; 128],
        }
    }

    /// Adds `mask · x = rhs`; `false` if it contradicts the equations so far.
    fn add(&mut self, mut mask: u128, mut rhs: bool) -> bool {
        while mask != 0 {
            let top = 127 - mask.leading_zeros() as usize;
            match self.pivots[top] {
                Some((m, r)) => {
                    mask ^= m;
                    rhs ^= r;
                }
                None => {
                    self.pivots[top] = Some((mask, rhs));
                    return true;
                }
            }
        }
        !rhs
    }

    fn is_complete(&self) -> bool {
        self.pivots.iter().all(Option::is_some)
    }

    /// The unique solution, if every unknown is determined.
    fn solve(&self) -> Option<u128> {
        let mut x = 0u128;
        for (top, row) in self.pivots.iter().enumerate() {
            let (mask, rhs) = (*row)?;
            let rest = mask & !(1u128 << top);
            if rhs ^ ((rest & x).count_ones() & 1 == 1) {
                x |= 1u128 << top;
            }
        }
        Some(x)
    }
}

/// Solves the generator for one window of keystream words.
///
/// Returns the untempered generator outputs `y[0..WINDOW + 4]`, where keystream word `t`
/// is made from outputs `t..=t + 4`.
fn solve_window(keystream: &[u32], untemper_images: &[u32; 32]) -> Option<Vec<u32>> {
    // Tempered outputs m[t]: four unknowns, then m[t + 4] from keystream word t.
    let mut m: Vec<Affine> = (0..4).map(Affine::unknown).collect();
    for (t, &k) in keystream.iter().enumerate() {
        let next = Affine::constant(k)
            .xor(&m[t + 3].rotate_right(13))
            .xor(&m[t + 2].rotate_right(6))
            .xor(&m[t + 1].rotate_left(5))
            .xor(&m[t].rotate_right(10));
        m.push(next);
    }
    // Untempering is costly in this representation, so only do it for words used.
    let mut y: Vec<Option<Affine>> = vec![None; m.len()];
    let mut untempered =
        |i: usize| *y[i].get_or_insert_with(|| m[i].map(untemper_images, untemper));
    let mut equations = Equations::new();
    for n in 0..m.len() - N {
        let (a, b) = (untempered(n), untempered(n + 1));
        let mut e = untempered(n + N).xor(&untempered(n + M));
        // ((a & UPPER) | (b & LOWER)) >> 1
        e.bits[30] ^= a.bits[31];
        for bit in 0..30 {
            e.bits[bit] ^= b.bits[bit + 1];
        }
        // ^ (b & 1) * MATRIX_A
        for bit in 0..32 {
            if MATRIX_A >> bit & 1 == 1 {
                e.bits[bit] ^= b.bits[0];
            }
        }
        e.constant ^= twist(a.constant, b.constant);
        for bit in 0..32 {
            if !equations.add(e.bits[bit], e.constant >> bit & 1 == 1) {
                return None;
            }
        }
    }
    if !equations.is_complete() {
        return None;
    }
    let x = equations.solve()?;
    Some(m.iter().map(|w| untemper(w.eval(x))).collect())
}

/// Runs the MT19937 recurrence backwards to fill `ys[..from]` from `ys[from..]`.
///
/// `ys[i]` gets its lower 31 bits from the step that produced `ys[i + 623]` and its top
/// bit from the step that produced `ys[i + 624]`.
fn run_backwards(ys: &mut [u32], from: usize) {
    let untwist = |v: u32| {
        // v = (y >> 1) ^ (y & 1) * MATRIX_A, where y's top bit came from the first word.
        if v & 0x8000_0000 != 0 {
            ((v ^ MATRIX_A) << 1) | 1
        } else {
            v << 1
        }
    };
    for i in (0..from).rev() {
        let lower = untwist(ys[i + N - 1] ^ ys[i + M - 1]) & 0x7fff_ffff;
        let upper = untwist(ys[i + N] ^ ys[i + M]) & 0x8000_0000;
        ys[i] = upper | lower;
    }
}

/// Inverts `init_by_array` on the state it produced and returns its 16-word key, if the
/// state really came from `init_by_array` with a 16-word key.
fn init_by_array_key(state: &[u32], seed_state: &[u32; N]) -> Option<[u32; KEY_WORDS]> {
    // Second loop, for i ≥ 3 (state[i - 1] already holds its final value there).
    let after_first = |i: usize| {
        let p = state[i - 1];
        state[i].wrapping_add(i as u32) ^ (p ^ (p >> 30)).wrapping_mul(INIT_MUL_2)
    };
    // First loop, for i ≥ 4: key[j] + j = after_first[i] - (seed[i] ^ f(after_first[i-1])).
    let key_at = |i: usize| {
        let p = after_first(i - 1);
        let j = (i - 1) % KEY_WORDS;
        after_first(i)
            .wrapping_sub(seed_state[i] ^ (p ^ (p >> 30)).wrapping_mul(INIT_MUL_1))
            .wrapping_sub(j as u32)
    };
    // Cheap rejection first: almost every candidate fails here.
    if key_at(4) != key_at(4 + KEY_WORDS) || key_at(5) != key_at(5 + KEY_WORDS) {
        return None;
    }
    let mut key = [0u32; KEY_WORDS];
    for i in 4..N {
        let v = key_at(i);
        let j = (i - 1) % KEY_WORDS;
        if i < 4 + KEY_WORDS {
            key[j] = v;
        } else if key[j] != v {
            return None;
        }
    }
    Some(key)
}

/// Rolling key from the block key of `data.dat`.
fn rolling_from_data_key(key: &[u32; KEY_WORDS]) -> [u8; 64] {
    let mut rolling = [0u8; 64];
    for (lane, pair) in rolling
        .as_chunks_mut::<8>()
        .0
        .iter_mut()
        .zip(key.as_chunks::<2>().0)
    {
        let v = ((u64::from(pair[1]) << 32) | u64::from(pair[0])) ^ DATA_BLOCK;
        *lane = v.to_le_bytes();
    }
    rolling
}

/// Searches `file` for the rolling key.
///
/// Blocks start at `first_block` at the earliest. Windows are tried from `scan_from` to
/// the end, then from `first_block` up to `scan_from`; put `scan_from` where long
/// constant areas are expected to find them quickly. Each candidate key is passed to
/// `accept`, which decides whether it really opens the file. If a block key is found
/// but rejected, the file is damaged rather than unknown, and the search stops.
pub(crate) fn find_rolling_key(
    file: &[u8],
    first_block: usize,
    scan_from: usize,
    mut accept: impl FnMut(&[u8; 64]) -> bool,
) -> Option<[u8; 64]> {
    let window_bytes = 4 * WINDOW + 3;
    let end = file.len().checked_sub(window_bytes)?;
    let scan_from = scan_from.clamp(first_block, end);
    let mut untemper_images = [0u32; 32];
    for (i, image) in untemper_images.iter_mut().enumerate() {
        *image = untemper(1 << i);
    }
    let seed_state = seeded_state(INIT_BY_ARRAY_SEED);
    let positions = (scan_from..end)
        .step_by(STRIDE)
        .chain((first_block..scan_from).step_by(STRIDE));
    let mut keystream = vec![0u32; WINDOW];
    let mut solved = 0;
    for pattern in PATTERNS {
        for position in positions.clone() {
            for offset in position..position + 4 {
                for (t, k) in keystream.iter_mut().enumerate() {
                    let at = offset + 4 * t;
                    let word = [file[at], file[at + 1], file[at + 2], file[at + 3]];
                    *k = u32::from_le_bytes(word) ^ pattern;
                }
                let Some(window) = solve_window(&keystream, &untemper_images) else {
                    continue;
                };
                solved += 1;
                if solved > MAX_SOLVED_WINDOWS {
                    return None;
                }
                // Candidate block starts: first_block + 4i ≡ offset (mod 4), at least
                // four words before the window (earlier words use a different formula).
                let Some(words_back) = offset.checked_sub(first_block).map(|d| d / 4) else {
                    continue;
                };
                if words_back < 4 {
                    continue;
                }
                let from = words_back + N;
                let mut ys = vec![0u32; from + window.len()];
                ys[from..].copy_from_slice(&window);
                run_backwards(&mut ys, from);
                // Block starting `j` words before the window: its first output is
                // ys[from - j], so init_by_array's state is ys[from - j - N..from - j].
                for j in (4..=words_back).rev() {
                    let start = from - j - N;
                    let Some(key) = init_by_array_key(&ys[start..start + N], &seed_state) else {
                        continue;
                    };
                    let rolling = rolling_from_data_key(&key);
                    return accept(&rolling).then_some(rolling);
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mt19937::Mt19937;

    #[test]
    fn init_by_array_is_inverted() {
        let key: [u32; KEY_WORDS] =
            std::array::from_fn(|i| (i as u32).wrapping_mul(0x9e37_79b9) ^ 0x5bd1);
        let state = Mt19937::from_key(&key).initial_state();
        let seed_state = seeded_state(INIT_BY_ARRAY_SEED);
        assert_eq!(init_by_array_key(&state, &seed_state), Some(key));
        let mut other = state;
        other[100] ^= 1;
        assert_eq!(init_by_array_key(&other, &seed_state), None);
    }

    #[test]
    fn generator_is_solved_and_run_backwards() {
        let mut mt = Mt19937::from_key(&[1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16]);
        let outputs: Vec<u32> = (0..3000).map(|_| mt.next_u32()).collect();
        // Keystream word t of a window starting at output s uses outputs s+t..=s+t+4.
        let s = 1500;
        let keystream: Vec<u32> = (0..WINDOW)
            .map(|t| {
                let o = |i: usize| outputs[s + t + i];
                o(4) ^ o(3).rotate_right(13)
                    ^ o(2).rotate_right(6)
                    ^ o(1).rotate_left(5)
                    ^ o(0).rotate_right(10)
            })
            .collect();
        let mut images = [0u32; 32];
        for (i, image) in images.iter_mut().enumerate() {
            *image = untemper(1 << i);
        }
        let window = solve_window(&keystream, &images).expect("solvable");
        let expected: Vec<u32> = outputs[s..s + WINDOW + 4]
            .iter()
            .map(|&o| untemper(o))
            .collect();
        assert_eq!(window, expected);

        let mut ys = vec![0u32; s + window.len()];
        ys[s..].copy_from_slice(&window);
        run_backwards(&mut ys, s);
        let all: Vec<u32> = outputs[..s].iter().map(|&o| untemper(o)).collect();
        assert_eq!(ys[..s], all[..]);

        let mut wrong = keystream.clone();
        wrong[300] ^= 1;
        assert!(solve_window(&wrong, &images).is_none());
    }
}
