# User guide

Installation: [Windows](INSTALL-WINDOWS.md) · [macOS](INSTALL-MACOS.md) · [Linux](INSTALL-LINUX.md)

> **Always quit PES 2021 before saving.** The editor refuses to write if the save changes
> while it is open, but the game may still overwrite your edit when it saves later.

## Desktop app

1. Click **Open save…** or drag an `ML0000000x` file onto the window. The header shows the
   club, league and in-game date so you can confirm it is the right slot.
2. Edit any of the fields below. Only fields you actually change are written to the file.
3. Keep **Back up before saving** checked and click **Save**.
4. The status bar confirms *"Saved and verified successfully"* and shows the backup path.

**Discard changes** reloads the values from the file. The interface opens in the
language of your system (Portuguese on Portuguese systems, Spanish on Spanish systems,
English on all others) and can be switched (English / Español / Português) at the top,
together with the display currency.

### Fields

| Field | Notes |
|---|---|
| **Manager name** | Up to 45 bytes of UTF-8 (≈ 45 Latin letters, fewer with accents or other scripts). Control and invisible characters are rejected. The editor finds *your* manager by the special ID the game gives to user-created managers, so it also works with patches that add coaches. If no such manager exists in the save the field is disabled. |
| **Transfer budget** | Shown in euros. The game stores money in units of €100, so values are rounded to the nearest hundred. |
| **Salary budget** | Same as the transfer budget. |
| **Club followers** | Any whole number up to 4,294,967,295. |
| **Maximum money** | Fills both budgets with €9,999,999,900. |
| **Currency** | Display only (US$ ×1.1, ¥ ×125, £ ×0.9, R$ ×4.3 — fixed approximate rates). The save is always written in euros. |

Budgets are limited to €99,999,999,900 to stay far from integer overflow in the game.

## Backups and restoring

Each save creates `ML0000000x.backup-YYYYMMDD-HHMMSS` (UTC time) in the same folder,
containing the file exactly as it was before the edit. To restore:

1. Quit the game.
2. Delete (or rename) `ML0000000x`.
3. Rename the backup back to `ML0000000x`.

Backups are never overwritten. Delete old ones whenever you like.

## Command line

The `pes21-ml` program does the same from a terminal. Paths with spaces need quotes.

```text
pes21-ml show <FILE> [--json] [--currency eur|usd|jpy|gbp|brl]
pes21-ml set  <FILE> [options]
pes21-ml decrypt <FILE> <NEW_FOLDER>
pes21-ml encrypt <FOLDER> <OUT_FILE> [--force]
```

### `set` options

| Option | Meaning |
|---|---|
| `--transfer <AMOUNT>` | Transfer budget in the chosen currency (thousands separators allowed: `500.000.000`, `500,000,000`). |
| `--salary <AMOUNT>` | Salary budget. |
| `--max-money` | Both budgets to €9,999,999,900. |
| `--followers <N>` | Club followers. |
| `--manager-name <NAME>` | Manager name. |
| `--currency <C>` | Currency of the amounts you type (default `eur`). |
| `--raw` | Amounts are raw stored values (units of €100) instead of euros. |
| `--dry-run` | Show what would change without writing. |
| `--no-backup` | Do not create a backup (not recommended). |

Examples:

```bash
pes21-ml set ML00000001 --max-money --dry-run
pes21-ml set ML00000001 --transfer 250.000.000 --followers 5000000
pes21-ml set ML00000001 --manager-name "José Mourinho"
pes21-ml show ML00000001 --json
```

`show --json` prints a single JSON object (`manager_name` is `null` when the manager is
not found). The exit code is `0` on success and `1` on any error.

### `decrypt` / `encrypt`

For advanced users and researchers: `decrypt` writes the decrypted blocks of any PES
2016–2021 save (`encryptHeader.dat`, `header.dat`, `description.dat`, `logo.png`,
`data.dat`, `version.txt`, plus `game.txt`) into a **new** folder; `encrypt` rebuilds a
save from such a folder. The layout is compatible with pesXdecrypter. The format is
described in [SAVE_FORMAT.md](SAVE_FORMAT.md).

## Error messages

| Message | Meaning |
|---|---|
| *invalid save layout or wrong key* / *not a known PES 2016–2021 save* | The file is not a PES save or is damaged. |
| *this save is from …, not eFootball PES 2021* | A save from another PES edition. |
| *not a compatible Master League save* | A PES 2021 file that is not a Master League save (e.g. `EDIT` or `SYSTEM`). |
| *re-encryption does not reproduce the file* | The editor does not fully understand this file, so it refuses to modify it. |
| *the player's manager was not found* | No user-created manager in this save; the name cannot be changed. |
| *the save changed on disk while editing* | The game or Steam Cloud wrote the file meanwhile. Quit the game and retry. |
| *post-save verification failed; previous contents restored* | Nothing was lost; please report it as a bug. |

## Compatibility

Tested with PES 2021 Season Update (exe 1.01–1.04, data packs 1.0–4.0) and with the
community patch "Football Life 2025". Other patches may move data around; always check
that the values shown match what you see in the game before saving.
