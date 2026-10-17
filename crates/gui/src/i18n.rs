//! User-interface strings.

use pes21_ml_core::master_league::NameProblem;
use pes21_ml_core::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Lang {
    PtBr,
    #[default]
    En,
    Es,
}

pub struct Texts {
    pub open: &'static str,
    pub language: &'static str,
    pub currency: &'static str,
    pub choose_file: &'static str,
    pub choosing: &'static str,
    pub welcome: &'static str,
    pub safety_note: &'static str,
    pub hint: &'static str,
    pub loaded: &'static str,
    pub untitled: &'static str,
    pub manager: &'static str,
    pub manager_not_found: &'static str,
    pub transfer: &'static str,
    pub salary: &'static str,
    pub followers: &'static str,
    pub max_money: &'static str,
    pub max_money_hint: &'static str,
    pub units_note: &'static str,
    pub backup: &'static str,
    pub no_backup_warning: &'static str,
    pub save: &'static str,
    pub revert: &'static str,
    pub close_game: &'static str,
    pub saved: &'static str,
    pub backup_at: &'static str,
    pub nothing_changed: &'static str,
    pub invalid_number: &'static str,
    pub too_large: &'static str,
    pub max_is: &'static str,
    pub errors: ErrorTexts,
}

/// Messages for [`Error`]. The core library's own messages are pt-BR / en in one string
/// (for the command line); the window shows them in the selected language instead.
/// `{}` marks where the value goes.
pub struct ErrorTexts {
    pub io: &'static str,
    pub too_large: &'static str,
    pub too_small: &'static str,
    pub bad_layout: &'static str,
    pub unknown_key: &'static str,
    pub wrong_game: &'static str,
    pub not_master_league: &'static str,
    pub round_trip: &'static str,
    pub out_of_range: &'static str,
    pub name_empty: &'static str,
    pub name_too_long: &'static str,
    pub name_control_chars: &'static str,
    pub invalid_path: &'static str,
    pub backup_name: &'static str,
    pub changed_on_disk: &'static str,
    pub verify_restored: &'static str,
    pub verify_failed: &'static str,
}

const PT_BR: Texts = Texts {
    open: "Abrir save…",
    language: "Idioma:",
    currency: "Moeda:",
    choose_file: "Escolha o arquivo ML (ex.: ML00000013)",
    choosing: "Escolha o arquivo na janela de seleção…",
    welcome: "Abra ou arraste aqui um arquivo MLxxxxxxxx da pasta\nDocumentos\\KONAMI\\eFootball PES 2021 SEASON UPDATE\\<seu ID>\\",
    safety_note: "Tudo é feito na memória, sem programas externos. Um backup é criado antes de salvar.",
    hint: "Feche o jogo antes de salvar.",
    loaded: "Carregado:",
    untitled: "Master League",
    manager: "Nome do técnico",
    manager_not_found: "técnico do jogador não encontrado neste save",
    transfer: "Orçamento de transferências",
    salary: "Orçamento salarial",
    followers: "Seguidores do clube",
    max_money: "Dinheiro máximo",
    max_money_hint: "Coloca os dois orçamentos em 9.999.999.900 €",
    units_note: "O save guarda os orçamentos em unidades de 100 €; valores são arredondados para a centena mais próxima. Outras moedas usam cotações fixas aproximadas, só para exibição.",
    backup: "Criar backup antes de salvar (recomendado)",
    no_backup_warning: "Sem backup não há como desfazer se algo der errado.",
    save: "Salvar",
    revert: "Descartar alterações",
    close_game: "Feche o PES 2021 antes de salvar.",
    saved: "Salvo e verificado com sucesso.",
    backup_at: "Backup:",
    nothing_changed: "Nada foi alterado.",
    invalid_number: "número inválido",
    too_large: "valor grande demais",
    max_is: "o máximo é",
    errors: ErrorTexts {
        io: "erro de arquivo: {}",
        too_large: "arquivo grande demais ({} bytes)",
        too_small: "arquivo pequeno demais ({} bytes) para ser um save",
        bad_layout: "estrutura do save inválida ou chave errada",
        unknown_key: "não é um save conhecido do PES 2016–2021",
        wrong_game: "este save é do {}, não do eFootball PES 2021",
        not_master_league: "não é um save de Master League compatível",
        round_trip: "a recriptografia não reproduz o arquivo lido; edição bloqueada por segurança",
        out_of_range: "valor fora do limite (máx. {})",
        name_empty: "nome do técnico: não pode ficar vazio",
        name_too_long: "nome do técnico: longo demais (máx. 45 bytes UTF-8)",
        name_control_chars: "nome do técnico: caracteres de controle ou invisíveis não são permitidos",
        invalid_path: "caminho inválido",
        backup_name: "não foi possível escolher um nome livre para o backup",
        changed_on_disk: "o save foi alterado por outro programa durante a edição; feche o jogo e tente de novo",
        verify_restored: "a verificação após salvar falhou e o conteúdo anterior foi restaurado",
        verify_failed: "a verificação após salvar falhou e não foi possível restaurar; use o backup",
    },
};

const EN: Texts = Texts {
    open: "Open save…",
    language: "Language:",
    currency: "Currency:",
    choose_file: "Choose the ML file (e.g. ML00000013)",
    choosing: "Pick the file in the file dialog…",
    welcome: "Open or drop here an MLxxxxxxxx file from\nDocuments\\KONAMI\\eFootball PES 2021 SEASON UPDATE\\<your ID>\\",
    safety_note: "Everything happens in memory, with no external programs. A backup is made before saving.",
    hint: "Close the game before saving.",
    loaded: "Loaded:",
    untitled: "Master League",
    manager: "Manager name",
    manager_not_found: "the player's manager was not found in this save",
    transfer: "Transfer budget",
    salary: "Salary budget",
    followers: "Club followers",
    max_money: "Maximum money",
    max_money_hint: "Sets both budgets to €9,999,999,900",
    units_note: "The save stores budgets in units of €100; amounts are rounded to the nearest hundred. Other currencies use fixed approximate rates, for display only.",
    backup: "Back up before saving (recommended)",
    no_backup_warning: "Without a backup there is no way back if something goes wrong.",
    save: "Save",
    revert: "Discard changes",
    close_game: "Close PES 2021 before saving.",
    saved: "Saved and verified successfully.",
    backup_at: "Backup:",
    nothing_changed: "Nothing changed.",
    invalid_number: "invalid number",
    too_large: "value too large",
    max_is: "the maximum is",
    errors: ErrorTexts {
        io: "file error: {}",
        too_large: "file too large ({} bytes)",
        too_small: "file too small ({} bytes) to be a save",
        bad_layout: "invalid save layout or wrong key",
        unknown_key: "not a known PES 2016–2021 save",
        wrong_game: "this save is from {}, not eFootball PES 2021",
        not_master_league: "not a compatible Master League save",
        round_trip: "re-encryption does not reproduce the file; editing blocked for safety",
        out_of_range: "value out of range (max {})",
        name_empty: "manager name: must not be empty",
        name_too_long: "manager name: too long (max 45 UTF-8 bytes)",
        name_control_chars: "manager name: control or invisible characters are not allowed",
        invalid_path: "invalid path",
        backup_name: "no free file name for the backup",
        changed_on_disk: "the save changed on disk while editing; close the game and try again",
        verify_restored: "post-save verification failed; the previous contents were restored",
        verify_failed: "post-save verification failed and restoring failed; use the backup",
    },
};

const ES: Texts = Texts {
    open: "Abrir partida guardada…",
    language: "Idioma:",
    currency: "Moneda:",
    choose_file: "Elige el archivo ML (p. ej., ML00000013)",
    choosing: "Elige el archivo en la ventana de selección…",
    welcome: "Abre o arrastra aquí un archivo MLxxxxxxxx de la carpeta\nDocumentos\\KONAMI\\eFootball PES 2021 SEASON UPDATE\\<tu ID>\\",
    safety_note: "Todo se hace en memoria, sin programas externos. Se crea una copia de seguridad antes de guardar.",
    hint: "Cierra el juego antes de guardar.",
    loaded: "Cargado:",
    untitled: "Liga Máster",
    manager: "Nombre del entrenador",
    manager_not_found: "no se encontró el entrenador del jugador en esta partida",
    transfer: "Presupuesto de fichajes",
    salary: "Presupuesto salarial",
    followers: "Seguidores del club",
    max_money: "Dinero máximo",
    max_money_hint: "Pone los dos presupuestos en 9.999.999.900 €",
    units_note: "La partida guarda los presupuestos en unidades de 100 €; los importes se redondean a la centena más cercana. Las demás monedas usan tipos de cambio fijos aproximados, solo para mostrar.",
    backup: "Crear copia de seguridad antes de guardar (recomendado)",
    no_backup_warning: "Sin copia de seguridad no hay forma de deshacer si algo sale mal.",
    save: "Guardar",
    revert: "Descartar cambios",
    close_game: "Cierra PES 2021 antes de guardar.",
    saved: "Guardado y verificado correctamente.",
    backup_at: "Copia de seguridad:",
    nothing_changed: "No se cambió nada.",
    invalid_number: "número no válido",
    too_large: "valor demasiado grande",
    max_is: "el máximo es",
    errors: ErrorTexts {
        io: "error de archivo: {}",
        too_large: "archivo demasiado grande ({} bytes)",
        too_small: "archivo demasiado pequeño ({} bytes) para ser una partida",
        bad_layout: "estructura de la partida no válida o clave incorrecta",
        unknown_key: "no es una partida conocida de PES 2016–2021",
        wrong_game: "esta partida es de {}, no de eFootball PES 2021",
        not_master_league: "no es una partida de Liga Máster compatible",
        round_trip: "volver a cifrar no reproduce el archivo; edición bloqueada por seguridad",
        out_of_range: "valor fuera de rango (máx. {})",
        name_empty: "nombre del entrenador: no puede estar vacío",
        name_too_long: "nombre del entrenador: demasiado largo (máx. 45 bytes UTF-8)",
        name_control_chars: "nombre del entrenador: no se permiten caracteres de control o invisibles",
        invalid_path: "ruta no válida",
        backup_name: "no se encontró un nombre libre para la copia de seguridad",
        changed_on_disk: "otro programa modificó la partida durante la edición; cierra el juego e inténtalo de nuevo",
        verify_restored: "la verificación tras guardar falló y se restauró el contenido anterior",
        verify_failed: "la verificación tras guardar falló y no se pudo restaurar; usa la copia de seguridad",
    },
};

impl Lang {
    pub const ALL: [Lang; 3] = [Lang::En, Lang::Es, Lang::PtBr];

    /// The operating system's language: Portuguese and Spanish variants get their
    /// language; every other language, or one that cannot be detected, gets English.
    pub fn from_system() -> Self {
        sys_locale::get_locale().map_or(Lang::En, |tag| Self::from_locale(&tag))
    }

    /// Maps a locale tag such as `pt-BR`, `es_ES.UTF-8` or `en-US` to a UI language.
    pub fn from_locale(tag: &str) -> Self {
        let language = tag.split(['-', '_', '.', '@']).next().unwrap_or_default();
        if language.eq_ignore_ascii_case("pt") {
            Lang::PtBr
        } else if language.eq_ignore_ascii_case("es") {
            Lang::Es
        } else {
            Lang::En
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Lang::PtBr => "Português (BR)",
            Lang::En => "English",
            Lang::Es => "Español",
        }
    }

    pub fn texts(self) -> &'static Texts {
        match self {
            Lang::PtBr => &PT_BR,
            Lang::En => &EN,
            Lang::Es => &ES,
        }
    }

    /// An [`Error`] from the core library, in this language.
    pub fn error(self, error: &Error) -> String {
        let t = self.texts();
        let e = &t.errors;
        let with = |template: &str, value: &dyn std::fmt::Display| {
            template.replacen("{}", &value.to_string(), 1)
        };
        match error {
            Error::Io(io) => with(e.io, io),
            Error::TooLarge(n) => with(e.too_large, n),
            Error::TooSmall(n) => with(e.too_small, n),
            Error::BadLayout => e.bad_layout.into(),
            Error::UnknownKey => e.unknown_key.into(),
            Error::WrongGame(game) => with(e.wrong_game, game),
            Error::NotMasterLeague => e.not_master_league.into(),
            Error::RoundTripMismatch => e.round_trip.into(),
            Error::OutOfRange { max, .. } => with(e.out_of_range, max),
            Error::InvalidName(NameProblem::Empty) => e.name_empty.into(),
            Error::InvalidName(NameProblem::TooLong) => e.name_too_long.into(),
            Error::InvalidName(NameProblem::ControlChars) => e.name_control_chars.into(),
            Error::ManagerNotFound => t.manager_not_found.into(),
            Error::InvalidPath => e.invalid_path.into(),
            Error::BackupNameUnavailable => e.backup_name.into(),
            Error::ChangedOnDisk => e.changed_on_disk.into(),
            Error::VerifyFailedRestored => e.verify_restored.into(),
            Error::VerifyFailed => e.verify_failed.into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn portuguese_locales_use_portuguese() {
        for tag in ["pt-BR", "pt_BR.UTF-8", "pt-PT", "PT", "pt", "pt_BR@euro"] {
            assert_eq!(Lang::from_locale(tag), Lang::PtBr, "{tag}");
        }
    }

    #[test]
    fn spanish_locales_use_spanish() {
        for tag in ["es-ES", "es_MX.UTF-8", "es-419", "ES", "es", "es_AR@euro"] {
            assert_eq!(Lang::from_locale(tag), Lang::Es, "{tag}");
        }
    }

    #[test]
    fn everything_else_falls_back_to_english() {
        for tag in [
            "en-US",
            "en_GB.UTF-8",
            "fr",
            "ja-JP",
            "C",
            "POSIX",
            "",
            "ptx",
            "est",
        ] {
            assert_eq!(Lang::from_locale(tag), Lang::En, "{tag}");
        }
        assert_eq!(Lang::default(), Lang::En);
    }

    #[test]
    fn core_errors_are_shown_in_one_language() {
        let err = Error::TooSmall(12);
        assert_eq!(
            Lang::Es.error(&err),
            "archivo demasiado pequeño (12 bytes) para ser una partida"
        );
        assert_eq!(
            Lang::En.error(&err),
            "file too small (12 bytes) to be a save"
        );
        assert_eq!(
            Lang::PtBr.error(&Error::InvalidName(NameProblem::Empty)),
            "nome do técnico: não pode ficar vazio"
        );
        // No message keeps an unfilled placeholder or the other language's half.
        for lang in Lang::ALL {
            for err in [
                Error::WrongGame("PES 2020"),
                Error::OutOfRange { field: "x", max: 9 },
            ] {
                let text = lang.error(&err);
                assert!(!text.contains("{}") && !text.contains(" / "), "{text}");
            }
        }
    }
}
