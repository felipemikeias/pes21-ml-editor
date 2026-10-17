# Installing on macOS

## Requirements

* macOS 11 Big Sur or newer
* A Mac with Apple Silicon (M1, M2, M3, M4…) **or** an Intel processor
* A PES 2021 Master League save. PES 2021 is a Windows game, so on a Mac it usually runs
  through CrossOver, Whisky, Parallels or a similar tool — the editor works on the save
  file wherever it is.

## 1. Pick the right download

Click the Apple menu → **About This Mac**:

| "Chip" or "Processor" shows | Download |
|---|---|
| Apple M1, M2, M3, M4… | `pes21-ml-editor-<version>-macos-aarch64.dmg` |
| Intel | `pes21-ml-editor-<version>-macos-x86_64.dmg` |

Optional — check the download in Terminal and compare with `SHA256SUMS.txt` from the
Releases page:

```bash
shasum -a 256 ~/Downloads/pes21-ml-editor-<version>-macos-aarch64.dmg
```

With the [GitHub CLI](https://cli.github.com/) you can also verify its build provenance:

```bash
gh attestation verify ~/Downloads/pes21-ml-editor-<version>-macos-aarch64.dmg --repo <owner>/pes21-ml-editor
```

## 2. Install

1. Double-click the `.dmg`.
2. Drag **PES 2021 ML Editor** onto the **Applications** shortcut.
3. Eject the disk image.

### First launch (Gatekeeper)

The app is not notarized by Apple, so macOS blocks the first launch:

1. Open **Applications**, right-click (or Control-click) **PES 2021 ML Editor** → **Open**.
2. Click **Open** in the dialog.

On macOS 15 and later, if there is no **Open** button: try to open the app once, then go to
**System Settings → Privacy & Security**, scroll down and click **Open Anyway**.

Only do this for a file whose hash you checked. From then on the app opens normally.

## 3. Find your save

The save lives inside the Windows environment the game runs in, under:

```
…/drive_c/users/<user>/Documents/KONAMI/eFootball PES 2021 SEASON UPDATE/<your ID>/ML0000000x
```

Examples:

* **CrossOver:** `~/Library/Application Support/CrossOver/Bottles/<bottle>/drive_c/users/crossover/Documents/KONAMI/…`
* **Whisky:** `~/Library/Containers/com.isaacmarovitz.Whisky/Bottles/<bottle>/drive_c/users/crossover/Documents/KONAMI/…`
* **Parallels:** the save is inside the Windows virtual machine; run the Windows version
  of the editor there instead.

`~/Library` is hidden in Finder: use **Go → Go to Folder…** (⇧⌘G) and paste the path.

## 4. Edit

1. **Quit PES 2021.**
2. Open the editor, click **Open save…** (or drag the `ML` file onto the window).
3. Change the values and click **Save**. A backup `ML0000000x.backup-YYYYMMDD-HHMMSS` is
   created next to the save first.

See [USAGE.md](USAGE.md) for every field and option.

## Command line (optional)

The disk image also contains `command-line/pes21-ml`, the terminal version:

```bash
mkdir -p ~/bin && cp "/Volumes/PES 2021 ML Editor/command-line/pes21-ml" ~/bin/
~/bin/pes21-ml show "/path/to/ML00000001"
```

If macOS blocks it, allow it once with `xattr -d com.apple.quarantine ~/bin/pes21-ml`.

## Building on a Mac

To make the app yourself instead of downloading it, install the Xcode command-line tools
(`xcode-select --install`) and Rust ([rustup.rs](https://rustup.rs)), then run from the
source folder:

```bash
scripts/build-macos.sh
```

This builds for the Mac you are on and writes `dist/PES 2021 ML Editor.app` and
`dist/pes21-ml-editor-<version>-macos-<arch>.dmg` (with a `.sha256` file). Options:

| Option | Effect |
|---|---|
| `--arch universal` | one app for Apple Silicon and Intel (about twice the size; needs `rustup target add x86_64-apple-darwin`) |
| `--arch aarch64` / `--arch x86_64` | build for the other architecture |
| `--no-dmg` | only the `.app` |
| `--out <folder>` | write somewhere other than `dist/` |

An app you built yourself is not quarantined, so it opens without the Gatekeeper steps.

## Uninstall

Drag **PES 2021 ML Editor** from Applications to the Trash (and delete `~/bin/pes21-ml`
if you copied it). The editor stores no settings elsewhere.
