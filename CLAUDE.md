# CLAUDE.md

Guidance for working on **pes21-ml-editor**: an open-source, cross-platform editor for
eFootball PES 2021 Master League saves (transfer/salary budgets, club followers, manager
name). Rust workspace, desktop app (egui) + command-line tool.

## Layout

| Path | What |
|---|---|
| `crates/core` (`pes21-ml-core`) | Library: save cipher (`crypto.rs`, `mt19937.rs`), Master League fields (`master_league.rs`), safe load/save (`save_file.rs`), display money (`money.rs`), bilingual `Error` (`lib.rs`) |
| `crates/cli` (`pes21-ml` binary) | `show`, `set`, `decrypt`, `encrypt` (clap) |
| `crates/gui` (`pes21-ml-editor` binary) | eframe/egui app; UI strings in `i18n.rs`; `build.rs` embeds Windows icon/version/manifest |
| `assets/` | `icon.svg` (source), `icon-256.png`, `icon.ico`, `icon.icns`, `linux/` (`.desktop`, `install.sh`), `macos/Info.plist` |
| `scripts/` | `build-macos.sh` (local `.app` + `.dmg`) |
| `docs/` | `INSTALL-*.md`, `USAGE.md`, `SAVE_FORMAT.md` (format spec — keep in sync with the code) |
| `.github/workflows/` | `ci.yml` (fmt, clippy, tests on 3 OSes, cargo audit), `release.yml` (packages for 4 targets) |

## Commands

```bash
cargo fmt --all --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked
# Integration tests against a real save (copied to a temp dir, never modified):
PES21_ML_SAMPLE=/path/to/ML0000000x cargo test -p pes21-ml-core --test real_save -- --include-ignored
cargo audit --deny warnings
cargo build --release --locked          # binaries in target/release/
```

MSRV is Rust 1.95 (`rust-version` in the workspace). Always build with `--locked`; commit
`Cargo.lock` changes together with `Cargo.toml` changes.

## Rules that must not be broken

* `unsafe_code = "forbid"` for the whole workspace. Do not add `unsafe` or FFI crates that
  require it in our code.
* No network access, telemetry or spawning of other programs. Everything runs in memory.
* Saves are personal data: never commit `ML0*` files or backups (see `.gitignore`), and
  never put names or values from a real save into tests or docs.
* Writing a save goes only through `save_file::save_edits`, which guarantees: input
  validated (block sizes add up, file type `ML`, re-encryption reproduces the file byte for
  byte), backup of the exact validated bytes (`create_new`), re-read check for concurrent
  changes, atomic write (temp file + `fsync` + rename) keeping permissions and following
  symlinks, read-back verification with automatic rollback.
* `master_league::Edit` writes **only** fields set to `Some`; untouched bytes must stay
  identical. New fields need: offset constant, validation in `Edit::validate`, a test
  proving no other byte changes, and an entry in `docs/SAVE_FORMAT.md`.
* Validate everything before writing anything (`Edit::apply` resolves and validates first).

## Save format essentials

* Cipher: port of the public-domain pesXdecrypter (MT19937-keyed XOR stream; per-game
  64-byte master keys in `crypto.rs`). Details in `docs/SAVE_FORMAT.md`.
* Budgets are `u32` in units of €100 at `0xC7DD00` / `0xC7DD14`; followers at `0xB629A4`.
* The manager is **not** at a fixed offset: it is the coach-table record (base name
  `0x133A38`, 600-byte records, 1300 slots) whose `coach_id` is `0xFFFFFFF5` and whose
  stored slot number matches its position. Patches shift it; never hard-code a slot.
  Name field is 46 bytes (max 45 UTF-8 bytes + NUL); the next 2 bytes belong to
  something else and must not be touched.

## GUI notes (eframe 0.37)

* `eframe::App` is implemented with `fn ui(&mut self, ui, frame)`; panels are
  `egui::Panel::top/bottom(..).show(ui, ..)`.
* The file dialog runs on a worker thread (Linux/Windows) so the window keeps responding;
  on macOS it must stay on the main thread. Keep it that way.
* `with_app_id("pes21-ml-editor")` must match `assets/linux/pes21-ml-editor.desktop`
  (Linux docks pick the icon from the installed desktop entry).
* Every user-visible string lives in `i18n.rs` (English, Spanish, pt-BR). Core error
  messages are pt-BR / en in one string for the command line; the GUI shows them in the
  selected language through `Lang::error`, so a new `Error` variant needs a text there in
  every language (the match is exhaustive, so the build fails until it has one).

## Packaging and releases

* `release.yml` runs on `v*` tags (draft release with `SHA256SUMS.txt` and provenance
  attestation) or manually via `workflow_dispatch` (artifacts only, named
  `dev-<commit>`). Packages: Windows `.zip`, macOS `.dmg` per architecture (`.app` with
  `AppIcon.icns`, ad-hoc signed), Linux `.tar.gz` with `install.sh`. The command-line
  tool always goes in a `command-line/` subfolder so users do not open it by mistake.
* Release builds set `PES21_REQUIRE_WINDOWS_RESOURCES=1` so a missing icon fails the build.
* Actions are pinned to commit SHAs (tag in a comment); Dependabot updates them.
* Local Windows test builds from Linux: `cargo zigbuild --target x86_64-pc-windows-gnu`
  (needs zig + cargo-zigbuild; the icon needs an rc compiler via `RC_<target>`).
  macOS builds can only be made on a Mac or by the release workflow; on a Mac,
  `scripts/build-macos.sh` produces the same `.app`/`.dmg` locally.
* `assets/macos/Info.plist` is the bundle metadata used by both `release.yml` and
  `scripts/build-macos.sh` (`@VERSION@` is substituted).
* Release profile is `opt-level = "s"` (smaller GUI) with `pes21-ml-core` kept at
  `opt-level = 3`: the cipher is the only hot path.

## Style

* Docs, comments and README in English. The GUI is trilingual: `Lang::from_system()` picks
  Portuguese for any `pt*` system locale, Spanish for any `es*` locale and English for
  everything else (English is the fallback when the locale cannot be read).
* Match the surrounding code: small modules, doc comments explaining *why*, tests next to
  the code (`#[cfg(test)]`), no `unwrap()` outside tests (clippy warns).
