//! Save-file container cipher used by Pro Evolution Soccer 2016 – eFootball PES 2021.
//!
//! Rust implementation of the algorithm documented by the public-domain
//! [pesXdecrypter](https://github.com/the4chancup/pesXdecrypter) project.
//!
//! A save file is laid out as:
//!
//! | block              | size                 | key                                   |
//! |--------------------|----------------------|---------------------------------------|
//! | encryption header  | 320                  | `header[256..320] ^ shuffled(master)` |
//! | file header        | 208 (PES 2018+)      | `rolling ^ u64(208)`                  |
//! | description        | `desc_size`          | `rolling ^ u64(0)`                    |
//! | logo (PNG)         | `logo_size`          | `rolling ^ u64(1)`                    |
//! | data               | `data_size`          | `rolling ^ u64(2)`                    |
//! | serial (UTF-16)    | `serial_len * 2`     | `rolling ^ u64(3)`                    |
//!
//! Every block is encrypted with [`crypt_in_place`], a XOR stream derived from an
//! MT19937 generator seeded with the 64-byte block key. XOR is symmetric, so the
//! same routine encrypts and decrypts.

use crate::mt19937::Mt19937;
use crate::Error;

/// Size of the leading encryption header.
pub const ENCRYPTION_HEADER_SIZE: usize = 320;
/// Size of the file header for PES 2018 and later (includes the game version string).
pub const FILE_HEADER_SIZE: usize = 208;
/// Size of the file header for PES 2016 and PES 2017.
pub const FILE_HEADER_SIZE_LEGACY: usize = 176;
/// Saves are a few dozen MiB at most; anything larger is rejected before allocating.
pub const MAX_SAVE_SIZE: usize = 256 * 1024 * 1024;

/// A 64-byte per-game master key.
pub type MasterKey = [u8; 64];

/// Games whose master keys are known (published by pesXdecrypter).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Game {
    Pes2016,
    Pes2016MyClub,
    Pes2017,
    Pes2018,
    Pes2019,
    Pes2020,
    Pes2021,
}

impl Game {
    pub const ALL: [Game; 7] = [
        Game::Pes2021,
        Game::Pes2020,
        Game::Pes2019,
        Game::Pes2018,
        Game::Pes2017,
        Game::Pes2016,
        Game::Pes2016MyClub,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Game::Pes2016 => "PES 2016",
            Game::Pes2016MyClub => "PES 2016 myClub",
            Game::Pes2017 => "PES 2017",
            Game::Pes2018 => "PES 2018",
            Game::Pes2019 => "PES 2019",
            Game::Pes2020 => "eFootball PES 2020",
            Game::Pes2021 => "eFootball PES 2021",
        }
    }

    pub fn file_header_size(self) -> usize {
        match self {
            Game::Pes2016 | Game::Pes2016MyClub | Game::Pes2017 => FILE_HEADER_SIZE_LEGACY,
            _ => FILE_HEADER_SIZE,
        }
    }

    pub fn master_key(self) -> &'static MasterKey {
        match self {
            Game::Pes2016 => &KEY_PES16,
            Game::Pes2016MyClub => &KEY_PES16_MYCLUB,
            Game::Pes2017 => &KEY_PES17,
            Game::Pes2018 => &KEY_PES18,
            Game::Pes2019 => &KEY_PES19,
            Game::Pes2020 => &KEY_PES20,
            Game::Pes2021 => &KEY_PES21,
        }
    }
}

#[rustfmt::skip]
const KEY_PES16: MasterKey = [
    0x4D, 0x55, 0x94, 0x66, 0xD9, 0x62, 0x5C, 0xEC, 0xC1, 0x7C, 0x48, 0x36, 0x77, 0x31, 0x50, 0xE1,
    0x87, 0x1C, 0xB5, 0x6B, 0x41, 0xD4, 0x92, 0x4F, 0x4A, 0x8C, 0x71, 0x27, 0x0A, 0x0D, 0x50, 0x63,
    0x94, 0x2B, 0x58, 0x5E, 0x99, 0x0B, 0x8B, 0x97, 0x96, 0x66, 0xC0, 0x00, 0xB7, 0x1D, 0x72, 0x75,
    0xD6, 0xE8, 0x5B, 0x0E, 0xAF, 0xF1, 0x72, 0xD1, 0xB1, 0xE3, 0x3C, 0x75, 0xDE, 0x9C, 0x13, 0x09,
];
#[rustfmt::skip]
const KEY_PES16_MYCLUB: MasterKey = [
    0x85, 0x91, 0x1E, 0x0F, 0x32, 0xE4, 0xBA, 0xD7, 0x45, 0x2C, 0xBC, 0xF1, 0x06, 0x16, 0x85, 0x94,
    0xCE, 0x43, 0x52, 0xF5, 0x41, 0x54, 0x87, 0x8B, 0xCB, 0x88, 0x46, 0xC6, 0x15, 0xB7, 0xB3, 0xDC,
    0x36, 0x1F, 0x13, 0x62, 0xF9, 0x03, 0xD4, 0x00, 0xAD, 0xD2, 0xE3, 0xC3, 0x91, 0xE6, 0x2F, 0x31,
    0x79, 0x66, 0xB5, 0x5A, 0x00, 0x9F, 0x93, 0x56, 0xB0, 0xB7, 0x43, 0x7C, 0x2F, 0x4C, 0x97, 0xAB,
];
#[rustfmt::skip]
const KEY_PES17: MasterKey = [
    0x9B, 0xC7, 0x13, 0x28, 0x2D, 0xE8, 0x47, 0x75, 0x4D, 0x52, 0x9E, 0x35, 0x90, 0xAA, 0x6A, 0x7A,
    0x5C, 0xFA, 0x60, 0x9F, 0x6A, 0x32, 0x04, 0x57, 0xB8, 0x9F, 0x59, 0xA5, 0x5F, 0xAC, 0x7D, 0x62,
    0xFE, 0x10, 0x2A, 0xD6, 0x95, 0xFA, 0xDF, 0xA0, 0x68, 0xBD, 0x40, 0x95, 0x47, 0x9C, 0xBB, 0x40,
    0xF2, 0x94, 0x49, 0x3C, 0xC8, 0xE0, 0x94, 0x9D, 0x7B, 0x01, 0x6F, 0xF2, 0xC5, 0x3A, 0x2C, 0xE5,
];
#[rustfmt::skip]
const KEY_PES18: MasterKey = [
    0x47, 0x51, 0x83, 0x38, 0x83, 0x9E, 0x85, 0x3A, 0xFA, 0x39, 0x08, 0xDC, 0x86, 0xAF, 0x0C, 0xB5,
    0xC4, 0x0E, 0xE4, 0xE4, 0x15, 0xE3, 0xCB, 0x64, 0x1B, 0x8A, 0xF1, 0xE5, 0x2C, 0xBD, 0x4D, 0x1B,
    0xA4, 0xD6, 0xA1, 0x97, 0x2C, 0x4B, 0xD1, 0xDA, 0xFA, 0xCD, 0x7E, 0xB4, 0x8A, 0x18, 0xC9, 0xE8,
    0xC5, 0x3E, 0x1A, 0x8B, 0x19, 0x45, 0x01, 0xF8, 0x8B, 0x85, 0x97, 0x8B, 0x78, 0x7B, 0x3C, 0x2E,
];
#[rustfmt::skip]
const KEY_PES19: MasterKey = [
    0xFD, 0x60, 0x4A, 0x3E, 0xFD, 0x69, 0x20, 0xD1, 0x93, 0x92, 0x37, 0xD7, 0x60, 0xD8, 0x30, 0xEE,
    0x65, 0x66, 0xFD, 0x6C, 0xE6, 0x9E, 0x48, 0xF8, 0x0A, 0x0D, 0xC1, 0x23, 0x7F, 0xAC, 0x89, 0x05,
    0x1D, 0xF8, 0x5A, 0x79, 0x10, 0x7E, 0xAD, 0x81, 0xAC, 0xAE, 0x9A, 0x6A, 0xAB, 0x16, 0xA6, 0x81,
    0xC2, 0xD2, 0x18, 0xC0, 0xF4, 0xE6, 0x5C, 0x27, 0x74, 0xF6, 0xC1, 0x9F, 0xF5, 0x01, 0x38, 0x72,
];
#[rustfmt::skip]
const KEY_PES20: MasterKey = [
    0xE2, 0xBF, 0x51, 0x07, 0x54, 0xE6, 0x21, 0x78, 0x2C, 0x5E, 0x8D, 0x33, 0x13, 0x7A, 0xC9, 0x15,
    0x99, 0x77, 0xD9, 0xA0, 0x1B, 0xC2, 0x95, 0xD9, 0xBB, 0x9B, 0xB1, 0x00, 0x84, 0x1C, 0xB3, 0x62,
    0xE5, 0x40, 0xD9, 0x56, 0x45, 0x5B, 0x7C, 0x7C, 0x4F, 0xF1, 0xDA, 0x26, 0xB4, 0x5A, 0x0C, 0x5C,
    0x4D, 0x6B, 0x9E, 0x98, 0x75, 0xA9, 0x39, 0x07, 0x4C, 0x4B, 0x55, 0xBD, 0x8E, 0x01, 0xA9, 0x31,
];
#[rustfmt::skip]
const KEY_PES21: MasterKey = [
    0x90, 0x61, 0xD8, 0x66, 0x43, 0x77, 0x24, 0xF8, 0x92, 0xBA, 0xB8, 0x71, 0x21, 0xC7, 0x60, 0x63,
    0xF0, 0x91, 0x9A, 0x7D, 0xED, 0x47, 0x80, 0xDE, 0x51, 0xF5, 0xDD, 0xD1, 0x08, 0xFE, 0x32, 0x84,
    0xF5, 0x09, 0x92, 0x00, 0xB2, 0x3E, 0x88, 0x9F, 0xEB, 0x24, 0x43, 0x05, 0x58, 0x76, 0x00, 0x22,
    0x9B, 0xFE, 0xEC, 0xF6, 0x50, 0x00, 0x29, 0xD3, 0x42, 0x75, 0x50, 0xB9, 0xEC, 0xD2, 0xF6, 0x75,
];

/// XORs `buf` with the keystream derived from `key`. Encrypting and decrypting are the same.
pub fn crypt_in_place(key: &[u8; 64], buf: &mut [u8]) {
    let mut words = [0u32; 16];
    for (w, chunk) in words.iter_mut().zip(key.as_chunks::<4>().0) {
        *w = u32::from_le_bytes(*chunk);
    }
    let mut mt = Mt19937::from_key(&words);
    let mut c0 = mt.next_u32();
    let mut c1 = mt.next_u32();
    let mut c2 = mt.next_u32();
    let mut c3 = mt.next_u32();

    let (chunks, rest) = buf.as_chunks_mut::<4>();
    for chunk in chunks {
        let c4 = mt.next_u32();
        *chunk = (u32::from_le_bytes(*chunk) ^ c4 ^ c3 ^ c2 ^ c1 ^ c0).to_le_bytes();
        c0 = c1.rotate_right(15);
        c1 = c2.rotate_left(11);
        c2 = c3.rotate_left(7);
        c3 = c4.rotate_right(13);
    }
    if !rest.is_empty() {
        let mut tail = [0u8; 4];
        tail[..rest.len()].copy_from_slice(rest);
        let v = u32::from_le_bytes(tail) ^ mt.next_u32() ^ c3 ^ c2 ^ c1 ^ c0;
        rest.copy_from_slice(&v.to_le_bytes()[..rest.len()]);
    }
}

/// Reverses the byte order inside each 8-byte lane of the master key.
fn shuffle_master_key(master: &MasterKey) -> [u8; 64] {
    let mut out = [0u8; 64];
    for (o, i) in out
        .as_chunks_mut::<8>()
        .0
        .iter_mut()
        .zip(master.as_chunks::<8>().0)
    {
        *o = *i;
        o.reverse();
    }
    out
}

/// En/decrypts the 320-byte encryption header. Bytes 256..320 are the per-save
/// random seed and are stored in clear text.
fn crypt_encryption_header(
    header: &[u8; ENCRYPTION_HEADER_SIZE],
    master: &MasterKey,
) -> [u8; ENCRYPTION_HEADER_SIZE] {
    let shuffled = shuffle_master_key(master);
    let mut key = [0u8; 64];
    key.copy_from_slice(&header[256..320]);
    for (k, s) in key.iter_mut().zip(shuffled.iter()) {
        *k ^= s;
    }
    let mut out = *header;
    crypt_in_place(&key, &mut out);
    out[256..320].copy_from_slice(&header[256..320]);
    out
}

/// Folds the decrypted encryption header into the 64-byte rolling key.
fn rolling_key(plain_header: &[u8; ENCRYPTION_HEADER_SIZE]) -> [u8; 64] {
    let mut key = [0u8; 64];
    key.copy_from_slice(&plain_header[..64]);
    for (i, b) in plain_header[64..].iter().enumerate() {
        key[i & 63] ^= b;
    }
    key
}

fn block_key(rolling: &[u8; 64], param: u64) -> [u8; 64] {
    let mut out = [0u8; 64];
    for (o, r) in out
        .as_chunks_mut::<8>()
        .0
        .iter_mut()
        .zip(rolling.as_chunks::<8>().0)
    {
        *o = (u64::from_le_bytes(*r) ^ param).to_le_bytes();
    }
    out
}

/// Decrypted contents of a save file. Field names follow pesXdecrypter's output files.
#[derive(Clone, PartialEq, Eq)]
pub struct Container {
    pub game: Game,
    /// `encryptHeader.dat` — plaintext encryption header (bytes 256..320 are the seed).
    pub encryption_header: [u8; ENCRYPTION_HEADER_SIZE],
    /// `header.dat` — file header. Its size fields are recomputed when encrypting.
    pub file_header: Vec<u8>,
    /// `description.dat`
    pub description: Vec<u8>,
    /// `logo.png`
    pub logo: Vec<u8>,
    /// `data.dat` — the actual game data.
    pub data: Vec<u8>,
    /// `version.txt` — UTF-16LE serial string.
    pub serial: Vec<u8>,
}

impl std::fmt::Debug for Container {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Container")
            .field("game", &self.game)
            .field("file_type", &self.file_type())
            .field("game_version", &self.game_version())
            .field("description", &self.description.len())
            .field("logo", &self.logo.len())
            .field("data", &self.data.len())
            .field("serial", &self.serial.len())
            .finish()
    }
}

fn read_u32(buf: &[u8], off: usize) -> u32 {
    u32::from_le_bytes([buf[off], buf[off + 1], buf[off + 2], buf[off + 3]])
}

fn c_string(bytes: &[u8]) -> String {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    String::from_utf8_lossy(&bytes[..end]).into_owned()
}

impl Container {
    /// File type tag from the header (`"ML"` for Master League saves).
    pub fn file_type(&self) -> String {
        self.file_header
            .get(144..176)
            .map(c_string)
            .unwrap_or_default()
    }

    /// Game version string (PES 2018+ only), e.g. `"eFootball PES 2021"`.
    pub fn game_version(&self) -> String {
        self.file_header
            .get(176..208)
            .map(c_string)
            .unwrap_or_default()
    }

    /// Human-readable lines of the description block (team, date, league...).
    pub fn description_text(&self) -> Vec<String> {
        self.description
            .split(|&b| b == 0)
            .filter(|s| !s.is_empty())
            .flat_map(|s| {
                String::from_utf8_lossy(s)
                    .lines()
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .filter(|s| !s.trim().is_empty())
            .collect()
    }

    /// Decrypts `file` with the given game's master key.
    ///
    /// The declared block sizes must add up to exactly the file length; this both
    /// rejects truncated/corrupted files and detects a wrong master key.
    pub fn decrypt(file: &[u8], game: Game) -> Result<Self, Error> {
        let fh_size = game.file_header_size();
        if file.len() > MAX_SAVE_SIZE {
            return Err(Error::TooLarge(file.len()));
        }
        if file.len() < ENCRYPTION_HEADER_SIZE + fh_size {
            return Err(Error::TooSmall(file.len()));
        }
        let mut enc = [0u8; ENCRYPTION_HEADER_SIZE];
        enc.copy_from_slice(&file[..ENCRYPTION_HEADER_SIZE]);
        let encryption_header = crypt_encryption_header(&enc, game.master_key());
        let rolling = rolling_key(&encryption_header);

        let mut pos = ENCRYPTION_HEADER_SIZE;
        let mut file_header = file[pos..pos + fh_size].to_vec();
        crypt_in_place(&block_key(&rolling, fh_size as u64), &mut file_header);
        pos += fh_size;

        let data_size = read_u32(&file_header, 64) as usize;
        let logo_size = read_u32(&file_header, 68) as usize;
        let desc_size = read_u32(&file_header, 72) as usize;
        let serial_size = (read_u32(&file_header, 76) as usize)
            .checked_mul(2)
            .ok_or(Error::BadLayout)?;
        let expected = [data_size, logo_size, desc_size, serial_size]
            .iter()
            .try_fold(pos, |acc, &s| acc.checked_add(s))
            .ok_or(Error::BadLayout)?;
        if expected != file.len() {
            return Err(Error::BadLayout);
        }

        let mut take = |len: usize, param: u64| {
            let mut block = file[pos..pos + len].to_vec();
            crypt_in_place(&block_key(&rolling, param), &mut block);
            pos += len;
            block
        };
        let description = take(desc_size, 0);
        let logo = take(logo_size, 1);
        let data = take(data_size, 2);
        let serial = take(serial_size, 3);

        Ok(Self {
            game,
            encryption_header,
            file_header,
            description,
            logo,
            data,
            serial,
        })
    }

    /// Tries every known master key and returns the first one whose layout checks out.
    pub fn decrypt_any(file: &[u8]) -> Result<Self, Error> {
        for game in Game::ALL {
            match Self::decrypt(file, game) {
                Ok(c) => return Ok(c),
                Err(Error::BadLayout) => continue,
                Err(e) => return Err(e),
            }
        }
        Err(Error::UnknownKey)
    }

    /// Re-encrypts the container. Header size fields are refreshed from the block lengths.
    pub fn encrypt(&self) -> Result<Vec<u8>, Error> {
        let fh_size = self.game.file_header_size();
        if self.file_header.len() != fh_size {
            return Err(Error::BadLayout);
        }
        if !self.serial.len().is_multiple_of(2) {
            return Err(Error::BadLayout);
        }
        let to_u32 = |n: usize| u32::try_from(n).map_err(|_| Error::BadLayout);
        let mut header = self.file_header.clone();
        header[64..68].copy_from_slice(&to_u32(self.data.len())?.to_le_bytes());
        header[68..72].copy_from_slice(&to_u32(self.logo.len())?.to_le_bytes());
        header[72..76].copy_from_slice(&to_u32(self.description.len())?.to_le_bytes());
        header[76..80].copy_from_slice(&to_u32(self.serial.len() / 2)?.to_le_bytes());

        let total = [
            self.description.len(),
            self.logo.len(),
            self.data.len(),
            self.serial.len(),
        ]
        .iter()
        .try_fold(ENCRYPTION_HEADER_SIZE + fh_size, |acc, &n| {
            acc.checked_add(n)
        })
        .ok_or(Error::TooLarge(usize::MAX))?;
        if total > MAX_SAVE_SIZE {
            return Err(Error::TooLarge(total));
        }
        let mut out = Vec::with_capacity(total);
        out.extend_from_slice(&crypt_encryption_header(
            &self.encryption_header,
            self.game.master_key(),
        ));
        let rolling = rolling_key(&self.encryption_header);
        for (block, param) in [
            (&header, fh_size as u64),
            (&self.description, 0),
            (&self.logo, 1),
            (&self.data, 2),
            (&self.serial, 3),
        ] {
            let start = out.len();
            out.extend_from_slice(block);
            crypt_in_place(&block_key(&rolling, param), &mut out[start..]);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(game: Game) -> Container {
        let mut encryption_header = [0u8; ENCRYPTION_HEADER_SIZE];
        for (i, b) in encryption_header.iter_mut().enumerate() {
            *b = (i as u8).wrapping_mul(37).wrapping_add(11);
        }
        let mut file_header = vec![0u8; game.file_header_size()];
        file_header[144..146].copy_from_slice(b"ML");
        Container {
            game,
            encryption_header,
            file_header,
            description: b"Team\nLeague\0".to_vec(),
            logo: vec![0x89, b'P', b'N', b'G', 1, 2, 3],
            data: (0..10_003u32).map(|i| (i * 7) as u8).collect(),
            serial: "S-1-5".encode_utf16().flat_map(u16::to_le_bytes).collect(),
        }
    }

    #[test]
    fn crypt_is_an_involution() {
        let key = [7u8; 64];
        let original: Vec<u8> = (0..1027u32).map(|i| i as u8).collect();
        let mut buf = original.clone();
        crypt_in_place(&key, &mut buf);
        assert_ne!(buf, original);
        crypt_in_place(&key, &mut buf);
        assert_eq!(buf, original);
    }

    #[test]
    fn encrypt_then_decrypt_round_trips() {
        for game in [Game::Pes2021, Game::Pes2017] {
            let c = sample(game);
            let bytes = c.encrypt().expect("encrypt");
            let back = Container::decrypt(&bytes, game).expect("decrypt");
            // decrypt reads back the refreshed size fields
            let mut expected = c.clone();
            expected.file_header = back.file_header.clone();
            assert_eq!(back, expected);
            assert_eq!(back.file_type(), "ML");
            assert_eq!(back.encrypt().expect("encrypt"), bytes);
        }
    }

    #[test]
    fn wrong_key_is_rejected_and_detection_works() {
        let bytes = sample(Game::Pes2021).encrypt().expect("encrypt");
        assert!(matches!(
            Container::decrypt(&bytes, Game::Pes2020),
            Err(Error::BadLayout)
        ));
        assert_eq!(
            Container::decrypt_any(&bytes).expect("detect").game,
            Game::Pes2021
        );
    }

    #[test]
    fn malformed_containers_do_not_panic() {
        let mut c = sample(Game::Pes2021);
        c.file_header.truncate(10);
        assert_eq!(c.file_type(), "");
        assert_eq!(c.game_version(), "");
        let _ = format!("{c:?}");
        assert!(matches!(c.encrypt(), Err(Error::BadLayout)));
        let mut odd = sample(Game::Pes2021);
        odd.serial.push(0);
        assert!(matches!(odd.encrypt(), Err(Error::BadLayout)));
    }

    #[test]
    fn truncated_files_are_rejected() {
        let bytes = sample(Game::Pes2021).encrypt().expect("encrypt");
        for len in [0, 100, 600, bytes.len() - 1] {
            assert!(Container::decrypt(&bytes[..len], Game::Pes2021).is_err());
        }
    }
}
