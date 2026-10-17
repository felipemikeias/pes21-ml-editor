# Installing on Linux

## Requirements

* x86-64 PC with glibc 2.35 or newer (Ubuntu 22.04+, Debian 12+, Fedora 36+, Arch, …)
* OpenGL 2.0+ and an X11 or Wayland desktop
* For the "Open save…" dialog: `xdg-desktop-portal` (installed by default on GNOME and KDE).
  Without it you can still drag the file onto the window or pass it as an argument.
* PES 2021 running through Steam Proton (or Wine) with a Master League save

## 1. Download

Download `pes21-ml-editor-<version>-linux-x86_64.tar.gz` from the **Releases** page and
check it against `SHA256SUMS.txt`:

```bash
cd ~/Downloads
sha256sum -c SHA256SUMS.txt --ignore-missing
```

Optionally verify the build provenance with the [GitHub CLI](https://cli.github.com/):

```bash
gh attestation verify pes21-ml-editor-<version>-linux-x86_64.tar.gz --repo <owner>/pes21-ml-editor
```

## 2. Install

Run it straight from the extracted folder:

```bash
tar -xzf pes21-ml-editor-<version>-linux-x86_64.tar.gz
cd pes21-ml-editor-<version>-linux-x86_64
./pes21-ml-editor
```

`pes21-ml-editor` is the desktop app. The `command-line/` folder holds `pes21-ml`, the
optional terminal version — it opens no window.

To get a menu entry and the app icon in the dock / task switcher, install it for your
user (no root needed):

```bash
./install.sh
```

This copies the programs to `~/.local/bin` and adds a menu entry and icon under
`~/.local/share`. Desktops such as GNOME only show the app's own icon in the dock for
installed apps; when run straight from the folder it gets a generic icon.

## 3. Find your save

With Steam Proton the save is inside the game's Proton prefix:

```
~/.local/share/Steam/steamapps/compatdata/1259970/pfx/drive_c/users/steamuser/Documents/KONAMI/eFootball PES 2021 SEASON UPDATE/<your ID>/ML0000000x
```

`1259970` is the Steam app ID of eFootball PES 2021 Season Update. If the game is on
another Steam library, replace `~/.local/share/Steam` with that library's path; Flatpak
Steam uses `~/.var/app/com.valvesoftware.Steam/.local/share/Steam`.

## 4. Edit

1. **Quit PES 2021.**
2. Open the editor and click **Open save…** (or drag the file onto the window, or run
   `pes21-ml-editor /path/to/ML00000001`).
3. Change the values and click **Save**. A backup `ML0000000x.backup-YYYYMMDD-HHMMSS` is
   created next to the save first.

See [USAGE.md](USAGE.md) for every field and option.

## Build from source

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh   # Rust 1.95+
git clone https://github.com/<owner>/pes21-ml-editor
cd pes21-ml-editor
cargo build --release --locked
./target/release/pes21-ml-editor
```

## Uninstall

```bash
./install.sh --uninstall
```

## Troubleshooting

| Problem | Fix |
|---|---|
| "Open save…" does nothing | Install `xdg-desktop-portal` plus the backend for your desktop (`-gtk`, `-gnome` or `-kde`), or drag the file onto the window. |
| Window does not open / OpenGL error | Install your GPU's Mesa or proprietary OpenGL driver. |
| `GLIBC_2.35 not found` | Your distribution is too old; build from source. |
