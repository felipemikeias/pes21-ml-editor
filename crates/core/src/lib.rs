//! Core library of **pes21-ml-editor**, a safe editor for eFootball PES 2021 Master
//! League save files.
//!
//! * [`crypto`] — the PES 2016–2021 save container cipher.
//! * [`master_league`] — offsets and validation of the editable Master League fields.
//! * [`save_file`] — loading, backing up, atomically writing and verifying saves.
//! * [`money`] — display conventions (units of €100, other currencies).
//!
//! ```no_run
//! use pes21_ml_core::{master_league::Edit, save_file::save_edits};
//! let report = save_edits("ML00000013".as_ref(), &Edit::max_money(), true)?;
//! println!("backup at {:?}", report.backup);
//! # Ok::<(), pes21_ml_core::Error>(())
//! ```

pub mod crypto;
pub mod master_league;
pub mod money;
mod mt19937;
pub mod save_file;

/// Everything that can go wrong. Messages are bilingual (pt-BR / en).
#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("erro de arquivo / file error: {0}")]
    Io(#[from] std::io::Error),
    #[error("arquivo grande demais ({0} bytes) / file too large")]
    TooLarge(usize),
    #[error("arquivo pequeno demais ({0} bytes) para ser um save / file too small to be a save")]
    TooSmall(usize),
    #[error("estrutura do save inválida ou chave errada / invalid save layout or wrong key")]
    BadLayout,
    #[error("não é um save conhecido do PES 2016–2021 / not a known PES 2016–2021 save")]
    UnknownKey,
    #[error("este save é do {0}, não do eFootball PES 2021 / this save is from {0}, not eFootball PES 2021")]
    WrongGame(&'static str),
    #[error("não é um save de Master League compatível / not a compatible Master League save")]
    NotMasterLeague,
    #[error("a recriptografia não reproduz o arquivo lido; edição bloqueada por segurança / re-encryption does not reproduce the file; editing blocked for safety")]
    RoundTripMismatch,
    #[error("valor fora do limite para {field} (máx. {max}) / value out of range for {field} (max {max})")]
    OutOfRange { field: &'static str, max: u32 },
    #[error("nome inválido: {0} / invalid name")]
    InvalidName(master_league::NameProblem),
    #[error("técnico do jogador não encontrado neste save / the player's manager was not found in this save")]
    ManagerNotFound,
    #[error("caminho inválido / invalid path")]
    InvalidPath,
    #[error("não foi possível escolher um nome livre para o backup / no free backup file name")]
    BackupNameUnavailable,
    #[error("o save foi alterado por outro programa durante a edição; feche o jogo e tente de novo / the save changed on disk while editing; close the game and retry")]
    ChangedOnDisk,
    #[error("a verificação após salvar falhou e o conteúdo anterior foi restaurado / post-save verification failed; previous contents restored")]
    VerifyFailedRestored,
    #[error("a verificação após salvar falhou e não foi possível restaurar; use o backup / post-save verification failed and restore failed; use the backup")]
    VerifyFailed,
}
