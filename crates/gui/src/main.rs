//! Desktop editor for eFootball PES 2021 Master League saves.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod i18n;

use std::path::{Path, PathBuf};
use std::sync::mpsc::{self, Receiver, TryRecvError};

use eframe::egui;
use i18n::{Lang, Texts};
use pes21_ml_core::master_league::{
    validate_manager_name, Edit, MasterLeague, HARD_MAX_BUDGET_RAW, MANAGER_NAME_MAX_BYTES,
    MAX_BUDGET_RAW,
};
use pes21_ml_core::money::{group_thousands, parse_amount, Currency};
use pes21_ml_core::save_file::{save_edits, MlSave};

/// Window/taskbar icon (the executable and app bundle get theirs at packaging time).
const ICON_PNG: &[u8] = include_bytes!("../../../assets/icon-256.png");

fn main() -> eframe::Result<()> {
    let mut viewport = egui::ViewportBuilder::default();
    match eframe::icon_data::from_png_bytes(ICON_PNG) {
        Ok(icon) => viewport = viewport.with_icon(icon),
        Err(e) => eprintln!("warning: could not load the window icon: {e}"),
    }
    let options = eframe::NativeOptions {
        viewport: viewport
            .with_title("PES 2021 ML Editor")
            // Must match `pes21-ml-editor.desktop` so Linux docks show the app icon.
            .with_app_id("pes21-ml-editor")
            .with_inner_size([560.0, 560.0])
            .with_min_inner_size([460.0, 480.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };
    let initial = std::env::args_os().nth(1).map(PathBuf::from);
    eframe::run_native(
        "pes21-ml-editor",
        options,
        Box::new(move |_cc| {
            let mut app = App {
                lang: Lang::from_system(),
                ..App::default()
            };
            if let Some(p) = initial {
                app.open(&p);
            }
            Ok(Box::new(app))
        }),
    )
}

/// Message in the status bar.
///
/// It records *what* happened rather than finished text, so it is rendered in the
/// language selected now, not the one that was active when it was set.
enum Status {
    Choosing,
    Loaded(PathBuf),
    NothingChanged,
    Saved {
        backup: Option<PathBuf>,
    },
    Invalid(Field, InputError),
    /// Error from the core library, shown in the selected language.
    Core(pes21_ml_core::Error),
}

/// How the status bar colours a message.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Tone {
    Info,
    Success,
    Error,
}

/// Form field a validation error refers to.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    Transfer,
    Salary,
    Followers,
}

/// Why a typed number was rejected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum InputError {
    InvalidNumber,
    TooLarge,
    /// Above [`HARD_MAX_BUDGET_RAW`]; the message shows the limit in the current currency.
    AboveMax,
}

/// What the window shows about the open save.
///
/// The decrypted save (~13 MB) is dropped right after loading: saving re-reads and
/// re-validates the file from disk anyway, so keeping it would only cost memory.
struct Loaded {
    title: Option<String>,
    details: String,
    values: MasterLeague,
}

impl Loaded {
    fn new(save: MlSave) -> Self {
        let c = &save.container;
        let desc = c.description_text();
        Self {
            title: desc.get(1).cloned(),
            details: format!(
                "{} · {}",
                c.game_version(),
                desc.get(2..).unwrap_or_default().join(" · ")
            ),
            values: save.values,
        }
    }
}

struct Form {
    transfer: String,
    salary: String,
    followers: String,
    name: String,
}

#[derive(Default)]
struct App {
    lang: Lang,
    currency: Currency,
    path: Option<PathBuf>,
    save: Option<Loaded>,
    form: Option<Form>,
    make_backup: bool,
    status: Option<Status>,
    /// Result of a file dialog running on another thread.
    picking: Option<Receiver<Option<PathBuf>>>,
}

impl App {
    fn t(&self) -> &'static Texts {
        self.lang.texts()
    }

    fn money_text(&self, raw: u32) -> String {
        group_thousands(self.currency.display(raw), '.')
    }

    fn fill_form(&mut self) {
        let Some(save) = &self.save else { return };
        let v = save.values.clone();
        self.form = Some(Form {
            transfer: self.money_text(v.transfer_budget),
            salary: self.money_text(v.salary_budget),
            followers: group_thousands(u64::from(v.club_followers), '.'),
            name: v.manager_name.unwrap_or_default(),
        });
    }

    fn open(&mut self, path: &Path) {
        match MlSave::load(path) {
            Ok(save) => {
                self.save = Some(Loaded::new(save));
                self.path = Some(path.to_path_buf());
                self.make_backup = true;
                self.fill_form();
                self.status = Some(Status::Loaded(path.to_path_buf()));
            }
            Err(e) => {
                self.status = Some(Status::Core(e));
            }
        }
    }

    /// Budget typed in the form, in raw units.
    fn parse_budget(&self, text: &str) -> Result<u32, InputError> {
        let amount = parse_amount(text).ok_or(InputError::InvalidNumber)?;
        let raw = self.currency.to_raw(amount).ok_or(InputError::TooLarge)?;
        if raw > HARD_MAX_BUDGET_RAW {
            return Err(InputError::AboveMax);
        }
        Ok(raw)
    }

    /// Builds an [`Edit`] containing only the fields that differ from the save.
    fn build_edit(&self) -> Result<Edit, Status> {
        let (Some(save), Some(form)) = (&self.save, &self.form) else {
            return Ok(Edit::default());
        };
        let v = &save.values;
        let mut edit = Edit::default();
        let transfer = self
            .parse_budget(&form.transfer)
            .map_err(|e| Status::Invalid(Field::Transfer, e))?;
        // Compare in display units so an untouched field never gets rewritten due to rounding.
        if form.transfer != self.money_text(v.transfer_budget) {
            edit.transfer_budget = Some(transfer);
        }
        let salary = self
            .parse_budget(&form.salary)
            .map_err(|e| Status::Invalid(Field::Salary, e))?;
        if form.salary != self.money_text(v.salary_budget) {
            edit.salary_budget = Some(salary);
        }
        let followers = parse_amount(&form.followers)
            .and_then(|n| u32::try_from(n).ok())
            .ok_or(Status::Invalid(Field::Followers, InputError::InvalidNumber))?;
        if followers != v.club_followers {
            edit.club_followers = Some(followers);
        }
        if v.manager_name.as_ref().is_some_and(|n| *n != form.name) {
            validate_manager_name(&form.name).map_err(Status::Core)?;
            edit.manager_name = Some(form.name.clone());
        }
        Ok(edit)
    }

    /// The status bar text in the current language, with how to colour it.
    fn status_message(&self) -> Option<(Tone, String)> {
        let t = self.t();
        let message = match self.status.as_ref()? {
            Status::Choosing => (Tone::Info, t.choosing.to_string()),
            Status::Loaded(path) => (Tone::Info, format!("{} {}", t.loaded, path.display())),
            Status::NothingChanged => (Tone::Info, t.nothing_changed.to_string()),
            Status::Saved { backup } => {
                let mut msg = t.saved.to_string();
                if let Some(b) = backup {
                    msg.push_str(&format!("\n{} {}", t.backup_at, b.display()));
                }
                (Tone::Success, msg)
            }
            Status::Invalid(field, error) => {
                let label = match field {
                    Field::Transfer => t.transfer,
                    Field::Salary => t.salary,
                    Field::Followers => t.followers,
                };
                let reason = match error {
                    InputError::InvalidNumber => t.invalid_number.to_string(),
                    InputError::TooLarge => t.too_large.to_string(),
                    InputError::AboveMax => format!(
                        "{} {} {}",
                        t.max_is,
                        self.currency.symbol(),
                        self.money_text(HARD_MAX_BUDGET_RAW)
                    ),
                };
                (Tone::Error, format!("{label}: {reason}"))
            }
            Status::Core(e) => (Tone::Error, self.lang.error(e)),
        };
        Some(message)
    }

    fn save_now(&mut self) {
        let Some(path) = self.path.clone() else {
            return;
        };
        let edit = match self.build_edit() {
            Ok(e) if e.is_empty() => {
                self.status = Some(Status::NothingChanged);
                return;
            }
            Ok(e) => e,
            Err(status) => {
                self.status = Some(status);
                return;
            }
        };
        match save_edits(&path, &edit, self.make_backup) {
            Ok(report) if !report.changed => {
                self.status = Some(Status::NothingChanged);
            }
            Ok(report) => {
                self.open(&path);
                self.status = Some(Status::Saved {
                    backup: report.backup,
                });
            }
            Err(e) => self.status = Some(Status::Core(e)),
        }
    }

    /// Shows the "open file" dialog without freezing the window.
    ///
    /// A blocking dialog on the UI thread stops the event loop; on Linux the desktop
    /// then reports the app as "not responding". The dialog therefore runs on a worker
    /// thread and the result is picked up by [`Self::poll_pick`]. macOS requires AppKit
    /// panels on the main thread, where they run modally and keep the window alive.
    fn pick_file(&mut self, ctx: &egui::Context) {
        if self.picking.is_some() {
            return;
        }
        let mut dialog = rfd::FileDialog::new().set_title(self.t().choose_file);
        if let Some(dir) = self.path.as_ref().and_then(|p| p.parent()) {
            dialog = dialog.set_directory(dir);
        }
        if cfg!(target_os = "macos") {
            if let Some(p) = dialog.pick_file() {
                self.open(&p);
            }
            return;
        }
        let (tx, rx) = mpsc::channel();
        let ctx = ctx.clone();
        std::thread::spawn(move || {
            let _ = tx.send(dialog.pick_file());
            ctx.request_repaint();
        });
        self.picking = Some(rx);
        self.status = Some(Status::Choosing);
    }

    fn poll_pick(&mut self) {
        let Some(rx) = &self.picking else { return };
        match rx.try_recv() {
            Err(TryRecvError::Empty) => {}
            Err(TryRecvError::Disconnected) | Ok(None) => {
                self.picking = None;
                self.status = None;
            }
            Ok(Some(path)) => {
                self.picking = None;
                self.open(&path);
            }
        }
    }
}

impl eframe::App for App {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        self.poll_pick();
        let dropped = ui
            .ctx()
            .input(|i| i.raw.dropped_files.first().map(|f| f.path().to_path_buf()));
        if let Some(p) = dropped {
            self.open(&p);
        }
        let t = self.t();

        egui::Panel::top("top").show(ui, |ui| {
            ui.horizontal(|ui| {
                let can_pick = self.picking.is_none();
                if ui
                    .add_enabled(can_pick, egui::Button::new(t.open))
                    .clicked()
                {
                    self.pick_file(ui.ctx());
                }
                ui.separator();
                ui.label(t.language);
                egui::ComboBox::from_id_salt("lang")
                    .selected_text(self.lang.label())
                    .show_ui(ui, |ui| {
                        for l in Lang::ALL {
                            ui.selectable_value(&mut self.lang, l, l.label());
                        }
                    });
                ui.label(t.currency);
                let before = self.currency;
                let pending = (
                    self.form
                        .as_ref()
                        .and_then(|f| self.parse_budget(&f.transfer).ok()),
                    self.form
                        .as_ref()
                        .and_then(|f| self.parse_budget(&f.salary).ok()),
                );
                egui::ComboBox::from_id_salt("currency")
                    .selected_text(self.currency.label())
                    .show_ui(ui, |ui| {
                        for c in Currency::ALL {
                            ui.selectable_value(&mut self.currency, c, c.label());
                        }
                    });
                if before != self.currency {
                    // Re-express whatever is typed in the new currency.
                    let (tr, sa) = pending;
                    let texts = (
                        tr.map(|r| self.money_text(r)),
                        sa.map(|r| self.money_text(r)),
                    );
                    if let Some(f) = &mut self.form {
                        if let Some(s) = texts.0 {
                            f.transfer = s;
                        }
                        if let Some(s) = texts.1 {
                            f.salary = s;
                        }
                    }
                }
            });
        });

        egui::Panel::bottom("status").show(ui, |ui| {
            match self.status_message() {
                Some((Tone::Info, s)) => {
                    ui.label(s);
                }
                Some((Tone::Success, s)) => {
                    ui.colored_label(egui::Color32::from_rgb(40, 160, 70), s);
                }
                Some((Tone::Error, s)) => {
                    ui.colored_label(egui::Color32::from_rgb(210, 60, 60), s);
                }
                None => {
                    ui.label(t.hint);
                }
            }
            ui.add_space(2.0);
        });

        egui::CentralPanel::default().show(ui, |ui| {
            let Some(save) = &self.save else {
                ui.vertical_centered(|ui| {
                    ui.add_space(80.0);
                    ui.heading("eFootball PES 2021 — Master League");
                    ui.add_space(12.0);
                    ui.label(t.welcome);
                    ui.add_space(12.0);
                    let can_pick = self.picking.is_none();
                    if ui
                        .add_enabled(can_pick, egui::Button::new(t.open))
                        .clicked()
                    {
                        self.pick_file(ui.ctx());
                    }
                    ui.add_space(24.0);
                    ui.small(t.safety_note);
                });
                return;
            };

            ui.heading(save.title.as_deref().unwrap_or(t.untitled));
            ui.label(&save.details);
            if let Some(p) = &self.path {
                ui.small(p.display().to_string());
            }
            ui.separator();

            let sym = self.currency.symbol();
            let mut max_clicked = false;
            let has_manager = save.values.manager_name.is_some();
            if let Some(form) = &mut self.form {
                egui::Grid::new("fields")
                    .num_columns(2)
                    .spacing([16.0, 10.0])
                    .show(ui, |ui| {
                        ui.label(t.manager);
                        ui.add_enabled(
                            has_manager,
                            egui::TextEdit::singleline(&mut form.name)
                                .char_limit(MANAGER_NAME_MAX_BYTES)
                                .hint_text(t.manager_not_found),
                        )
                        .on_disabled_hover_text(t.manager_not_found);
                        ui.end_row();

                        ui.label(format!("{} ({sym})", t.transfer));
                        ui.text_edit_singleline(&mut form.transfer);
                        ui.end_row();

                        ui.label(format!("{} ({sym})", t.salary));
                        ui.text_edit_singleline(&mut form.salary);
                        ui.end_row();

                        ui.label(t.followers);
                        ui.text_edit_singleline(&mut form.followers);
                        ui.end_row();
                    });
                ui.add_space(8.0);
                max_clicked = ui
                    .button(t.max_money)
                    .on_hover_text(t.max_money_hint)
                    .clicked();
            }
            if max_clicked {
                let text = self.money_text(MAX_BUDGET_RAW);
                if let Some(f) = &mut self.form {
                    f.transfer = text.clone();
                    f.salary = text;
                }
            }

            ui.add_space(8.0);
            ui.small(t.units_note);
            ui.separator();

            ui.checkbox(&mut self.make_backup, t.backup);
            if !self.make_backup {
                ui.colored_label(egui::Color32::from_rgb(210, 140, 30), t.no_backup_warning);
            }
            ui.add_space(6.0);
            ui.horizontal(|ui| {
                if ui
                    .add(egui::Button::new(t.save).min_size(egui::vec2(140.0, 32.0)))
                    .clicked()
                {
                    self.save_now();
                }
                if ui.button(t.revert).clicked() {
                    self.fill_form();
                    self.status = None;
                }
            });
            ui.add_space(6.0);
            ui.small(t.close_game);
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_icon_decodes() {
        let icon = eframe::icon_data::from_png_bytes(super::ICON_PNG).expect("icon PNG decodes");
        assert_eq!((icon.width, icon.height), (256, 256));
    }

    #[test]
    fn status_follows_the_selected_language() {
        let mut app = App {
            lang: Lang::PtBr,
            status: Some(Status::Loaded(PathBuf::from("ML00000001"))),
            ..App::default()
        };
        let message = |app: &App| app.status_message().expect("status is set");
        assert_eq!(message(&app), (Tone::Info, "Carregado: ML00000001".into()));
        app.lang = Lang::En;
        assert_eq!(message(&app), (Tone::Info, "Loaded: ML00000001".into()));
        app.lang = Lang::Es;
        assert_eq!(message(&app), (Tone::Info, "Cargado: ML00000001".into()));
        app.lang = Lang::En;

        app.status = Some(Status::Invalid(Field::Followers, InputError::InvalidNumber));
        assert_eq!(
            message(&app),
            (Tone::Error, "Club followers: invalid number".into())
        );
        app.lang = Lang::PtBr;
        assert_eq!(
            message(&app),
            (Tone::Error, "Seguidores do clube: número inválido".into())
        );

        app.status = Some(Status::Core(pes21_ml_core::Error::ChangedOnDisk));
        app.lang = Lang::Es;
        assert_eq!(
            message(&app),
            (
                Tone::Error,
                "otro programa modificó la partida durante la edición; cierra el juego e inténtalo de nuevo".into()
            )
        );
    }
}
