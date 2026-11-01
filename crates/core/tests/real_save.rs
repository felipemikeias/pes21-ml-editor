//! Tests against a real save (eFootball PES 2021 or a game built on it, such as SP
//! Football Life 2027). Saves are personal data and are not committed, so these tests
//! only run when `PES21_ML_SAMPLE` points to an ML file:
//!
//! ```sh
//! PES21_ML_SAMPLE=/path/to/ML00000013 cargo test -p pes21-ml-core --test real_save -- --include-ignored
//! ```
//! The sample is copied to a temporary folder; the original is never modified.

use std::path::PathBuf;

use pes21_ml_core::crypto::{Container, Game};
use pes21_ml_core::master_league::{self, Edit, MasterLeague};
use pes21_ml_core::save_file::{read_limited, save_edits, MlSave, RECOVERY_SCAN_FROM};

fn sample() -> Option<PathBuf> {
    std::env::var_os("PES21_ML_SAMPLE").map(PathBuf::from)
}

#[test]
#[ignore = "needs PES21_ML_SAMPLE"]
fn real_save_round_trips_exactly() {
    let Some(path) = sample() else { return };
    let bytes = read_limited(&path).expect("read sample");
    let c = Container::decrypt_or_recover(&bytes, RECOVERY_SCAN_FROM).expect("decrypt");
    assert_eq!(c.file_type(), "ML");
    assert_eq!(c.encrypt().expect("encrypt"), bytes);
    let save = MlSave::from_bytes(&bytes).expect("validate");
    assert!(master_league::has_pes2021_layout(&c.data));
    println!("{:?}\n{:?}\n{:?}", c, c.description_text(), save.values);
}

/// Recovering the key from the file alone gives the same result as the known key, and
/// the master key the game stores in the file header is the one in [`Game`].
#[test]
#[ignore = "needs PES21_ML_SAMPLE"]
fn real_save_key_is_recovered_from_the_file() {
    let Some(path) = sample() else { return };
    let bytes = read_limited(&path).expect("read sample");
    let started = std::time::Instant::now();
    let recovered = Container::recover(&bytes, RECOVERY_SCAN_FROM).expect("recover");
    println!("recovered in {:?}", started.elapsed());
    let known = Container::decrypt_any(&bytes).expect("known key");
    let game: Game = known.game.expect("known game");
    assert_eq!(recovered.master_key, *game.master_key());
    assert_eq!(recovered.encryption_header, known.encryption_header);
    assert_eq!(recovered.data, known.data);
    assert_eq!(recovered.encrypt().expect("encrypt"), bytes);
    // Starting the search at the beginning of the file must find it too.
    assert_eq!(
        Container::recover(&bytes, 0).expect("recover").data,
        known.data
    );
}

#[test]
#[ignore = "needs PES21_ML_SAMPLE"]
fn real_save_edit_changes_only_target_bytes() {
    let Some(path) = sample() else { return };
    let dir = tempfile::tempdir().expect("tempdir");
    let copy = dir.path().join("ML00000099");
    std::fs::copy(&path, &copy).expect("copy");
    let before = MlSave::load(&copy).expect("load");

    let edit = Edit {
        transfer_budget: Some(master_league::MAX_BUDGET_RAW),
        salary_budget: Some(55_555_555),
        club_followers: Some(before.values.club_followers + 1),
        manager_name: Some("Teste Open Source".into()),
    };
    let report = save_edits(&copy, &edit, true).expect("save");
    let backup = report.backup.expect("backup created");
    assert_eq!(
        std::fs::read(&backup).expect("backup"),
        std::fs::read(&path).expect("orig")
    );

    let after = MlSave::load(&copy).expect("reload");
    assert_eq!(after.values.transfer_budget, master_league::MAX_BUDGET_RAW);
    assert_eq!(after.values.salary_budget, 55_555_555);
    assert_eq!(
        after.values.manager_name.as_deref(),
        Some("Teste Open Source")
    );

    // Only the bytes of the four fields may differ inside data.dat; every other block is identical.
    let (a, b) = (&before.container, &after.container);
    assert_eq!(a.encryption_header, b.encryption_header);
    assert_eq!(a.file_header, b.file_header);
    assert_eq!(
        (&a.description, &a.logo, &a.serial),
        (&b.description, &b.logo, &b.serial)
    );
    let owned = [
        master_league::TRANSFER_BUDGET.offset..master_league::TRANSFER_BUDGET.offset + 4,
        master_league::SALARY_BUDGET.offset..master_league::SALARY_BUDGET.offset + 4,
        master_league::CLUB_FOLLOWERS.offset..master_league::CLUB_FOLLOWERS.offset + 4,
    ];
    let owned: Vec<_> = owned
        .into_iter()
        .chain(
            master_league::manager_name_offsets(&a.data)
                .into_iter()
                .map(|o| o..o + master_league::MANAGER_NAME_LEN),
        )
        .collect();
    let diffs: Vec<usize> = a
        .data
        .iter()
        .zip(&b.data)
        .enumerate()
        .filter(|(_, (x, y))| x != y)
        .map(|(i, _)| i)
        .collect();
    assert!(!diffs.is_empty());
    for i in diffs {
        assert!(
            owned.iter().any(|r| r.contains(&i)),
            "unexpected change at {i:#x}"
        );
    }

    // Reverting the edit reproduces the original file exactly.
    let revert = Edit {
        transfer_budget: Some(before.values.transfer_budget),
        salary_budget: Some(before.values.salary_budget),
        club_followers: Some(before.values.club_followers),
        manager_name: before.values.manager_name.clone(),
    };
    save_edits(&copy, &revert, false).expect("revert");
    assert_eq!(
        std::fs::read(&copy).expect("copy"),
        std::fs::read(&path).expect("orig")
    );
    let _: MasterLeague = after.values;
}
