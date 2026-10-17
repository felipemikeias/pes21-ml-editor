//! Loading and saving Master League files safely.
//!
//! Guarantees of [`save_edits`]:
//!
//! * everything happens in memory, in-process — no helper executables, no temp folder;
//! * a save is only accepted if re-encrypting it unchanged reproduces the file byte for
//!   byte, proving the cipher and layout are understood for that file;
//! * the backup contains exactly the bytes that were validated, and never overwrites an
//!   existing file;
//! * if the file changes on disk while editing (e.g. the game is still running), nothing
//!   is written;
//! * the new file is written to a temporary file in the same folder, flushed to disk and
//!   atomically renamed over the save, keeping its permissions; symbolic links are
//!   followed so the real file is updated;
//! * the written file is read back and verified; on failure the previous contents are
//!   restored automatically.

use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::crypto::{Container, Game, MAX_SAVE_SIZE};
use crate::master_league::{Edit, MasterLeague};
use crate::Error;

/// A decrypted, validated Master League save.
#[derive(Debug, Clone)]
pub struct MlSave {
    pub container: Container,
    pub values: MasterLeague,
}

/// Reads a file, refusing anything larger than [`MAX_SAVE_SIZE`].
pub fn read_limited(path: &Path) -> Result<Vec<u8>, Error> {
    let file = fs::File::open(path)?;
    let len = file.metadata()?.len();
    if len > MAX_SAVE_SIZE as u64 {
        return Err(Error::TooLarge(usize::try_from(len).unwrap_or(usize::MAX)));
    }
    let mut buf = Vec::with_capacity(len as usize);
    file.take(MAX_SAVE_SIZE as u64 + 1).read_to_end(&mut buf)?;
    if buf.len() > MAX_SAVE_SIZE {
        return Err(Error::TooLarge(buf.len()));
    }
    Ok(buf)
}

impl MlSave {
    /// Decrypts and validates a PES 2021 Master League save held in memory.
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, Error> {
        let container = match Container::decrypt(bytes, Game::Pes2021) {
            Ok(c) => c,
            Err(Error::BadLayout) => {
                // Give a precise message if it is a save from another PES edition.
                return Err(match Container::decrypt_any(bytes) {
                    Ok(other) => Error::WrongGame(other.game.name()),
                    Err(_) => Error::UnknownKey,
                });
            }
            Err(e) => return Err(e),
        };
        if container.file_type() != "ML" {
            return Err(Error::NotMasterLeague);
        }
        // The encoder must reproduce the exact input before we allow writing anything.
        if container.encrypt()? != bytes {
            return Err(Error::RoundTripMismatch);
        }
        let values = MasterLeague::read(&container.data)?;
        Ok(Self { container, values })
    }

    /// Loads a save from disk.
    pub fn load(path: &Path) -> Result<Self, Error> {
        Self::from_bytes(&read_limited(path)?)
    }

    /// Applies `edit` to an in-memory copy and returns the new encrypted bytes.
    pub fn render(&self, edit: &Edit) -> Result<Vec<u8>, Error> {
        let mut container = self.container.clone();
        edit.apply(&mut container.data)?;
        container.encrypt()
    }
}

/// What [`save_edits`] did.
#[derive(Debug, Clone)]
pub struct SaveReport {
    /// The file that was actually written (symbolic links resolved).
    pub path: PathBuf,
    pub backup: Option<PathBuf>,
    /// `false` when the edit produced identical bytes and nothing was written.
    pub changed: bool,
    pub values: MasterLeague,
}

/// Current time formatted as `YYYYMMDD-HHMMSS` (UTC).
fn timestamp() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let (days, rem) = (secs / 86_400, secs % 86_400);
    // Civil-from-days (Howard Hinnant), valid for the proleptic Gregorian calendar.
    let z = days as i64 + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!(
        "{year:04}{month:02}{day:02}-{:02}{:02}{:02}",
        rem / 3600,
        rem % 3600 / 60,
        rem % 60
    )
}

/// Opens a brand-new file (fails if anything, including a symlink, already exists).
/// On Unix it starts private (0600) until the final permissions are applied.
fn create_new_file(path: &Path) -> std::io::Result<fs::File> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(path)
}

/// Writes `data` to `<name>.backup-<timestamp>[-n]` next to `path`, never overwriting
/// an existing file. The backup gets `permissions` (normally those of the save).
pub fn create_backup(
    path: &Path,
    data: &[u8],
    permissions: Option<&fs::Permissions>,
) -> Result<PathBuf, Error> {
    let name = path
        .file_name()
        .ok_or(Error::InvalidPath)?
        .to_string_lossy()
        .into_owned();
    let stamp = timestamp();
    for n in 0..1000 {
        let candidate = if n == 0 {
            path.with_file_name(format!("{name}.backup-{stamp}"))
        } else {
            path.with_file_name(format!("{name}.backup-{stamp}-{n}"))
        };
        match create_new_file(&candidate) {
            Ok(mut f) => {
                f.write_all(data)?;
                f.sync_all()?;
                if let Some(p) = permissions {
                    fs::set_permissions(&candidate, p.clone())?;
                }
                return Ok(candidate);
            }
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(e.into()),
        }
    }
    Err(Error::BackupNameUnavailable)
}

/// Atomically replaces `path` with `bytes` (temp file in the same folder + rename).
/// New files are private on Unix (0600); pass `permissions` to keep an existing file's.
pub fn write_atomically(
    path: &Path,
    bytes: &[u8],
    permissions: Option<&fs::Permissions>,
) -> Result<(), Error> {
    let dir = match path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let mut tmp = tempfile::Builder::new()
        .prefix(".pes21-ml-")
        .suffix(".tmp")
        .tempfile_in(dir)?;
    tmp.write_all(bytes)?;
    tmp.as_file().sync_all()?;
    if let Some(p) = permissions {
        fs::set_permissions(tmp.path(), p.clone())?;
    }
    tmp.persist(path).map_err(|e| Error::Io(e.error))?;
    Ok(())
}

/// Resolves a symbolic link so the real file is replaced instead of the link.
fn resolve_target(path: &Path) -> Result<PathBuf, Error> {
    if fs::symlink_metadata(path)?.file_type().is_symlink() {
        Ok(fs::canonicalize(path)?)
    } else {
        Ok(path.to_path_buf())
    }
}

fn verify_written(path: &Path, expected: &[u8], edit: &Edit) -> Result<MasterLeague, Error> {
    if read_limited(path)? != expected {
        return Err(Error::VerifyFailed);
    }
    let v = MlSave::from_bytes(expected)?.values;
    let ok = edit.transfer_budget.is_none_or(|x| x == v.transfer_budget)
        && edit.salary_budget.is_none_or(|x| x == v.salary_budget)
        && edit.club_followers.is_none_or(|x| x == v.club_followers)
        && edit
            .manager_name
            .as_ref()
            .is_none_or(|x| v.manager_name.as_ref() == Some(x));
    if ok {
        Ok(v)
    } else {
        Err(Error::VerifyFailed)
    }
}

/// Loads `path`, applies `edit`, backs the validated bytes up (if `backup`), writes the
/// result atomically and verifies it, restoring the previous contents if verification
/// fails.
pub fn save_edits(path: &Path, edit: &Edit, backup: bool) -> Result<SaveReport, Error> {
    let path = resolve_target(path)?;
    let original = read_limited(&path)?;
    let permissions = fs::metadata(&path)?.permissions();
    let save = MlSave::from_bytes(&original)?;
    let new_bytes = save.render(edit)?;
    if new_bytes == original {
        return Ok(SaveReport {
            path,
            backup: None,
            changed: false,
            values: save.values,
        });
    }

    let backup = if backup {
        Some(create_backup(&path, &original, Some(&permissions))?)
    } else {
        None
    };

    // Something else (usually the game) wrote the save after we read it.
    if read_limited(&path)? != original {
        return Err(Error::ChangedOnDisk);
    }
    write_atomically(&path, &new_bytes, Some(&permissions))?;

    match verify_written(&path, &new_bytes, edit) {
        Ok(values) => Ok(SaveReport {
            path,
            backup,
            changed: true,
            values,
        }),
        Err(_) => match write_atomically(&path, &original, Some(&permissions)) {
            Ok(()) => Err(Error::VerifyFailedRestored),
            Err(_) => Err(Error::VerifyFailed),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::{ENCRYPTION_HEADER_SIZE, FILE_HEADER_SIZE};
    use crate::master_league::{
        COACH_RECORD_SIZE, COACH_TABLE_FIRST_NAME, MIN_DATA_LEN, USER_COACH_ID,
    };

    /// A synthetic but structurally valid PES 2021 ML save.
    fn fixture() -> Vec<u8> {
        let mut file_header = vec![0u8; FILE_HEADER_SIZE];
        file_header[144..146].copy_from_slice(b"ML");
        let mut data = vec![0u8; MIN_DATA_LEN + 16];
        let name = COACH_TABLE_FIRST_NAME + 3 * COACH_RECORD_SIZE;
        data[name - 8..name - 4].copy_from_slice(&3u32.to_le_bytes());
        data[name - 4..name].copy_from_slice(&USER_COACH_ID.to_le_bytes());
        data[name..name + 4].copy_from_slice(b"Test");
        Container {
            game: Game::Pes2021,
            encryption_header: [3u8; ENCRYPTION_HEADER_SIZE],
            file_header,
            description: b"Team\0".to_vec(),
            logo: vec![1, 2, 3],
            data,
            serial: vec![b'S', 0],
        }
        .encrypt()
        .expect("fixture")
    }

    fn followers(n: u32) -> Edit {
        Edit {
            club_followers: Some(n),
            ..Edit::default()
        }
    }

    #[test]
    fn timestamp_shape() {
        let t = timestamp();
        assert_eq!(t.len(), 15);
        assert_eq!(&t[8..9], "-");
        assert!(t[..8].chars().all(|c| c.is_ascii_digit()));
    }

    #[test]
    fn save_backs_up_exact_bytes_and_verifies() {
        let dir = tempfile::tempdir().expect("tempdir");
        let save = dir.path().join("ML00000001");
        let original = fixture();
        fs::write(&save, &original).expect("write");

        let report = save_edits(&save, &followers(42), true).expect("save");
        assert!(report.changed);
        assert_eq!(report.values.club_followers, 42);
        let backup = report.backup.expect("backup");
        assert_eq!(fs::read(&backup).expect("backup"), original);
        assert_eq!(MlSave::load(&save).expect("load").values.club_followers, 42);

        // Only the save and its backup remain: no temporary files left behind.
        assert_eq!(fs::read_dir(dir.path()).expect("dir").count(), 2);
    }

    #[test]
    fn unchanged_edit_writes_nothing() {
        let dir = tempfile::tempdir().expect("tempdir");
        let save = dir.path().join("ML00000001");
        fs::write(&save, fixture()).expect("write");
        let report = save_edits(&save, &followers(0), true).expect("save");
        assert!(!report.changed);
        assert!(report.backup.is_none());
        assert_eq!(fs::read_dir(dir.path()).expect("dir").count(), 1);
    }

    #[test]
    fn invalid_files_are_left_untouched() {
        let dir = tempfile::tempdir().expect("tempdir");
        let save = dir.path().join("ML00000001");
        let mut bytes = fixture();
        bytes.pop();
        fs::write(&save, &bytes).expect("write");
        assert!(save_edits(&save, &followers(1), true).is_err());
        assert_eq!(fs::read(&save).expect("read"), bytes);
        assert_eq!(fs::read_dir(dir.path()).expect("dir").count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn keeps_permissions_and_follows_symlinks() {
        use std::os::unix::fs::PermissionsExt;
        let dir = tempfile::tempdir().expect("tempdir");
        let real = dir.path().join("ML00000001");
        fs::write(&real, fixture()).expect("write");
        fs::set_permissions(&real, fs::Permissions::from_mode(0o640)).expect("chmod");
        let link = dir.path().join("link");
        std::os::unix::fs::symlink(&real, &link).expect("symlink");

        let report = save_edits(&link, &followers(7), true).expect("save");
        assert!(fs::symlink_metadata(&link)
            .expect("link")
            .file_type()
            .is_symlink());
        assert_eq!(MlSave::load(&real).expect("load").values.club_followers, 7);
        let mode = |p: &Path| fs::metadata(p).expect("meta").permissions().mode() & 0o777;
        assert_eq!(mode(&real), 0o640);
        assert_eq!(mode(&report.backup.expect("backup")), 0o640);
    }
}
