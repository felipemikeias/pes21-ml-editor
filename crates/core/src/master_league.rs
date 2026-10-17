//! Master League fields inside the decrypted `data.dat` block.
//!
//! Offsets are the ones used by community Master League tools for PES 2021 (game exe
//! 1.01–1.04, data packs 1.0–4.0); see `docs/SAVE_FORMAT.md`.

use crate::Error;

/// A little-endian `u32` stored at a fixed offset of `data.dat`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct U32Field {
    pub offset: usize,
    pub name: &'static str,
}

/// Transfer budget, in units of €100.
pub const TRANSFER_BUDGET: U32Field = U32Field {
    offset: 0x00C7_DD00,
    name: "transfer_budget",
};
/// Salary budget, in units of €100.
pub const SALARY_BUDGET: U32Field = U32Field {
    offset: 0x00C7_DD14,
    name: "salary_budget",
};
/// Number of club followers.
pub const CLUB_FOLLOWERS: U32Field = U32Field {
    offset: 0x00B6_29A4,
    name: "club_followers",
};

/// Coaches live in a fixed table of [`COACH_SLOTS`] records of [`COACH_RECORD_SIZE`]
/// bytes. Each record starts with `u32 slot`, `u32 coach_id`, then the name. This is the
/// offset of the name of slot 0.
pub const COACH_TABLE_FIRST_NAME: usize = 0x0013_3A38;
pub const COACH_RECORD_SIZE: usize = 0x258;
pub const COACH_SLOTS: usize = 1300;
/// Bytes between the start of a record and its name.
const COACH_HEADER_SIZE: usize = 8;
/// Coach ID the game gives to the manager the player created for Master League.
/// The player's manager is found by this ID rather than by a fixed slot, because
/// database updates (official or community patches) add coaches and move it.
pub const USER_COACH_ID: u32 = 0xFFFF_FFF5;

/// Width of a coach name. Real saves keep a non-zero byte at offset +46, so the field is
/// 46 bytes wide (the width PES 2019–2021 also use for player names) and the next two
/// bytes belong to something else. This editor never touches them.
pub const MANAGER_NAME_LEN: usize = 46;
/// Longest name that still leaves room for the terminating NUL.
pub const MANAGER_NAME_MAX_BYTES: usize = MANAGER_NAME_LEN - 1;

/// "Maximum money" preset: €9,999,999,900, the conventional cap used by PES 2021 ML
/// editors.
pub const MAX_BUDGET_RAW: u32 = 99_999_999;
/// Hard limit accepted by this editor. Real saves exceed [`MAX_BUDGET_RAW`] on their own
/// (budgets above 100,000,000 have been observed), so that is not a game limit; this
/// bound only keeps the value far from `i32::MAX` in case the game treats it as signed
/// and keeps adding revenue to it.
pub const HARD_MAX_BUDGET_RAW: u32 = 999_999_999;

/// Smallest `data.dat` that contains every field above.
pub const MIN_DATA_LEN: usize = SALARY_BUDGET.offset + 4;

impl U32Field {
    pub fn read(&self, data: &[u8]) -> Result<u32, Error> {
        let b = data
            .get(self.offset..self.offset + 4)
            .ok_or(Error::NotMasterLeague)?;
        Ok(u32::from_le_bytes([b[0], b[1], b[2], b[3]]))
    }

    pub fn write(&self, data: &mut [u8], value: u32) -> Result<(), Error> {
        let b = data
            .get_mut(self.offset..self.offset + 4)
            .ok_or(Error::NotMasterLeague)?;
        b.copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
}

fn read_u32(data: &[u8], offset: usize) -> u32 {
    u32::from_le_bytes([
        data[offset],
        data[offset + 1],
        data[offset + 2],
        data[offset + 3],
    ])
}

/// The bytes of a name field up to the first NUL.
fn name_bytes(data: &[u8], offset: usize) -> &[u8] {
    let raw = &data[offset..offset + MANAGER_NAME_LEN];
    &raw[..raw.iter().position(|&b| b == 0).unwrap_or(raw.len())]
}

/// Offsets of the name fields that hold the player's manager name.
///
/// A record counts only if its coach ID is [`USER_COACH_ID`] *and* its stored slot
/// number matches its position, so random bytes are never mistaken for a record. The
/// game may keep more than one record for the manager; all records sharing the first
/// one's name are returned so they are renamed together.
pub fn manager_name_offsets(data: &[u8]) -> Vec<usize> {
    if data.len() < MIN_DATA_LEN {
        return Vec::new();
    }
    let user_records: Vec<usize> = (0..COACH_SLOTS)
        .map(|slot| (slot, COACH_TABLE_FIRST_NAME + slot * COACH_RECORD_SIZE))
        .filter(|&(slot, name)| {
            let header = name - COACH_HEADER_SIZE;
            read_u32(data, header) as usize == slot && read_u32(data, header + 4) == USER_COACH_ID
        })
        .map(|(_, name)| name)
        .collect();
    let Some(&first) = user_records.first() else {
        return Vec::new();
    };
    let wanted = name_bytes(data, first);
    user_records
        .into_iter()
        .filter(|&o| name_bytes(data, o) == wanted)
        .collect()
}

/// Current Master League values read from a save.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MasterLeague {
    /// Raw stored value, in units of €100.
    pub transfer_budget: u32,
    /// Raw stored value, in units of €100.
    pub salary_budget: u32,
    pub club_followers: u32,
    /// `None` if the save has no manager record (see [`manager_name_offsets`]).
    pub manager_name: Option<String>,
}

impl MasterLeague {
    /// Reads all fields from a decrypted `data.dat`.
    pub fn read(data: &[u8]) -> Result<Self, Error> {
        if data.len() < MIN_DATA_LEN {
            return Err(Error::NotMasterLeague);
        }
        let manager_name = manager_name_offsets(data)
            .first()
            .map(|&o| String::from_utf8_lossy(name_bytes(data, o)).into_owned());
        Ok(Self {
            transfer_budget: TRANSFER_BUDGET.read(data)?,
            salary_budget: SALARY_BUDGET.read(data)?,
            club_followers: CLUB_FOLLOWERS.read(data)?,
            manager_name,
        })
    }
}

/// A set of changes. Fields left as `None` are not written at all, so their bytes
/// stay exactly as the game wrote them.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Edit {
    pub transfer_budget: Option<u32>,
    pub salary_budget: Option<u32>,
    pub club_followers: Option<u32>,
    pub manager_name: Option<String>,
}

/// Zero-width and bidirectional-override characters: they render as nothing (or reorder
/// text) and are a classic way to make two names look identical.
fn is_invisible_format_char(c: char) -> bool {
    matches!(c,
        '\u{00AD}' | '\u{061C}' | '\u{180E}' | '\u{200B}'..='\u{200F}' | '\u{202A}'..='\u{202E}'
        | '\u{2060}'..='\u{2069}' | '\u{FEFF}')
}

/// Why a manager name was rejected. Kept as a value (not text) so front ends can
/// show it in their own language.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameProblem {
    Empty,
    TooLong,
    ControlChars,
}

impl std::fmt::Display for NameProblem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            NameProblem::Empty => "o nome não pode ficar vazio / name must not be empty",
            NameProblem::TooLong => {
                "nome longo demais (máx. 45 bytes UTF-8) / name too long (max 45 UTF-8 bytes)"
            }
            NameProblem::ControlChars => {
                "caracteres de controle ou invisíveis não são permitidos / control or invisible characters are not allowed"
            }
        })
    }
}

/// Checks a manager name before it is written.
pub fn validate_manager_name(name: &str) -> Result<(), Error> {
    if name.trim().is_empty() {
        return Err(Error::InvalidName(NameProblem::Empty));
    }
    if name.len() > MANAGER_NAME_MAX_BYTES {
        return Err(Error::InvalidName(NameProblem::TooLong));
    }
    if name
        .chars()
        .any(|c| c.is_control() || is_invisible_format_char(c))
    {
        return Err(Error::InvalidName(NameProblem::ControlChars));
    }
    Ok(())
}

impl Edit {
    /// Both budgets set to [`MAX_BUDGET_RAW`].
    pub fn max_money() -> Self {
        Self {
            transfer_budget: Some(MAX_BUDGET_RAW),
            salary_budget: Some(MAX_BUDGET_RAW),
            ..Self::default()
        }
    }

    pub fn is_empty(&self) -> bool {
        *self == Self::default()
    }

    /// Validates every requested change without touching any data.
    pub fn validate(&self) -> Result<(), Error> {
        for (field, value) in [
            (TRANSFER_BUDGET, self.transfer_budget),
            (SALARY_BUDGET, self.salary_budget),
        ] {
            if value.is_some_and(|v| v > HARD_MAX_BUDGET_RAW) {
                return Err(Error::OutOfRange {
                    field: field.name,
                    max: HARD_MAX_BUDGET_RAW,
                });
            }
        }
        if let Some(name) = &self.manager_name {
            validate_manager_name(name)?;
        }
        Ok(())
    }

    /// Applies the changes to a decrypted `data.dat`. Nothing is written unless every
    /// change is valid.
    pub fn apply(&self, data: &mut [u8]) -> Result<(), Error> {
        if data.len() < MIN_DATA_LEN {
            return Err(Error::NotMasterLeague);
        }
        self.validate()?;
        // Resolve everything before writing, so a failure leaves `data` untouched.
        let name_offsets = match &self.manager_name {
            Some(_) => {
                let offsets = manager_name_offsets(data);
                if offsets.is_empty() {
                    return Err(Error::ManagerNotFound);
                }
                offsets
            }
            None => Vec::new(),
        };
        if let Some(v) = self.transfer_budget {
            TRANSFER_BUDGET.write(data, v)?;
        }
        if let Some(v) = self.salary_budget {
            SALARY_BUDGET.write(data, v)?;
        }
        if let Some(v) = self.club_followers {
            CLUB_FOLLOWERS.write(data, v)?;
        }
        if let Some(name) = &self.manager_name {
            for offset in name_offsets {
                let field = &mut data[offset..offset + MANAGER_NAME_LEN];
                field.fill(0);
                field[..name.len()].copy_from_slice(name.as_bytes());
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn name_at(slot: usize) -> usize {
        COACH_TABLE_FIRST_NAME + slot * COACH_RECORD_SIZE
    }

    fn put_coach(d: &mut [u8], slot: usize, id: u32, name: &str) {
        let o = name_at(slot);
        d[o - 8..o - 4].copy_from_slice(&(slot as u32).to_le_bytes());
        d[o - 4..o].copy_from_slice(&id.to_le_bytes());
        d[o..o + MANAGER_NAME_LEN].fill(0);
        d[o..o + name.len()].copy_from_slice(name.as_bytes());
    }

    /// Data with a database coach in slot 661 (where unpatched saves keep the manager),
    /// the player's manager twice in slots 721/722 and another user record in 723.
    fn sample() -> Vec<u8> {
        let mut d = vec![0xAAu8; MIN_DATA_LEN + 64];
        put_coach(&mut d, 661, 102_288, "Database Coach");
        put_coach(&mut d, 721, USER_COACH_ID, "Zé Técnico");
        put_coach(&mut d, 722, USER_COACH_ID, "Zé Técnico");
        put_coach(&mut d, 723, USER_COACH_ID, "Outro");
        // a user ID in a record whose slot number does not match is ignored
        put_coach(&mut d, 724, USER_COACH_ID, "Falso");
        let o = name_at(724);
        d[o - 8..o - 4].copy_from_slice(&0u32.to_le_bytes());
        d
    }

    fn owned(i: usize) -> bool {
        [
            TRANSFER_BUDGET.offset..TRANSFER_BUDGET.offset + 4,
            SALARY_BUDGET.offset..SALARY_BUDGET.offset + 4,
            CLUB_FOLLOWERS.offset..CLUB_FOLLOWERS.offset + 4,
            name_at(721)..name_at(721) + MANAGER_NAME_LEN,
            name_at(722)..name_at(722) + MANAGER_NAME_LEN,
        ]
        .iter()
        .any(|r| r.contains(&i))
    }

    #[test]
    fn finds_the_players_manager_not_a_database_coach() {
        let data = sample();
        assert_eq!(
            manager_name_offsets(&data),
            vec![name_at(721), name_at(722)]
        );
        assert_eq!(
            MasterLeague::read(&data)
                .expect("read")
                .manager_name
                .as_deref(),
            Some("Zé Técnico")
        );
    }

    #[test]
    fn edits_round_trip_and_touch_only_owned_bytes() {
        let mut data = sample();
        let before = data.clone();
        let edit = Edit {
            transfer_budget: Some(12_345_678),
            salary_budget: Some(MAX_BUDGET_RAW),
            club_followers: Some(4_000_000),
            manager_name: Some("José Mourinho".into()),
        };
        edit.apply(&mut data).expect("apply");
        let ml = MasterLeague::read(&data).expect("read");
        assert_eq!(ml.transfer_budget, 12_345_678);
        assert_eq!(ml.salary_budget, MAX_BUDGET_RAW);
        assert_eq!(ml.club_followers, 4_000_000);
        assert_eq!(ml.manager_name.as_deref(), Some("José Mourinho"));
        for (i, (a, b)) in before.iter().zip(&data).enumerate() {
            assert!(a == b || owned(i), "byte {i:#x} changed outside a field");
        }
        assert_eq!(name_bytes(&data, name_at(722)), "José Mourinho".as_bytes());
        assert_eq!(name_bytes(&data, name_at(723)), b"Outro");
        assert_eq!(name_bytes(&data, name_at(661)), b"Database Coach");
        assert_eq!(
            data[name_at(721) + 46],
            0xAA,
            "bytes after the name are preserved"
        );
    }

    #[test]
    fn missing_manager_is_reported_and_blocks_name_edits() {
        let mut data = vec![0xAAu8; MIN_DATA_LEN + 64];
        put_coach(&mut data, 661, 102_288, "Database Coach");
        assert_eq!(MasterLeague::read(&data).expect("read").manager_name, None);
        let before = data.clone();
        let edit = Edit {
            club_followers: Some(1),
            manager_name: Some("Novo".into()),
            ..Edit::default()
        };
        assert!(matches!(edit.apply(&mut data), Err(Error::ManagerNotFound)));
        assert_eq!(
            data, before,
            "nothing is written when the manager is missing"
        );
    }

    #[test]
    fn unset_fields_are_not_written() {
        let mut data = sample();
        let before = data.clone();
        Edit {
            club_followers: Some(1),
            ..Edit::default()
        }
        .apply(&mut data)
        .expect("apply");
        for (i, (a, b)) in before.iter().zip(&data).enumerate() {
            assert!(a == b || (CLUB_FOLLOWERS.offset..CLUB_FOLLOWERS.offset + 4).contains(&i));
        }
    }

    #[test]
    fn invalid_edits_change_nothing() {
        let mut data = sample();
        let before = data.clone();
        for edit in [
            Edit {
                transfer_budget: Some(HARD_MAX_BUDGET_RAW + 1),
                ..Edit::default()
            },
            Edit {
                club_followers: Some(5),
                manager_name: Some("x".repeat(46)),
                ..Edit::default()
            },
            Edit {
                manager_name: Some("a\nb".into()),
                ..Edit::default()
            },
            Edit {
                manager_name: Some("Jos\u{202E}e".into()),
                ..Edit::default()
            },
            Edit {
                manager_name: Some("Ana\u{200B}".into()),
                ..Edit::default()
            },
            Edit {
                manager_name: Some("   ".into()),
                ..Edit::default()
            },
        ] {
            assert!(edit.apply(&mut data).is_err());
            assert_eq!(data, before);
        }
        assert!(Edit {
            manager_name: Some("x".repeat(45)),
            ..Edit::default()
        }
        .apply(&mut data)
        .is_ok());
        assert!(MasterLeague::read(&data[..100]).is_err());
    }
}
