# Installing on Windows

## Requirements

* Windows 10 or 11, 64-bit
* A graphics driver with OpenGL 2.0 or newer (any GPU from the last decade)
* eFootball PES 2021 (Steam) with a Master League save

## 1. Download

1. Open the project's **Releases** page on GitHub.
2. Download `pes21-ml-editor-<version>-windows-x86_64.zip`.
3. Optional but recommended — check the download. In PowerShell:

   ```powershell
   Get-FileHash .\pes21-ml-editor-<version>-windows-x86_64.zip -Algorithm SHA256
   ```

   The hash must match the line for that file in `SHA256SUMS.txt` (also on the Releases
   page). With the [GitHub CLI](https://cli.github.com/) you can also verify that the file
   was built by the project's workflow:

   ```powershell
   gh attestation verify .\pes21-ml-editor-<version>-windows-x86_64.zip --repo <owner>/pes21-ml-editor
   ```

## 2. Install

The editor is portable: there is no installer and nothing is written to the registry.

1. Right-click the zip → **Extract All…** and pick a folder, for example
   `C:\Users\<you>\Apps\pes21-ml-editor`.
2. Open the folder and double-click **`pes21-ml-editor.exe`** (the desktop app). The
   `command-line` folder only contains the optional terminal version.
3. The first time, Windows SmartScreen may say *"Windows protected your PC"* because the
   app is not code-signed. Click **More info → Run anyway**. Only do this for a file whose
   hash you checked in step 1.

Optional: right-click `pes21-ml-editor.exe` → **Show more options → Send to → Desktop
(create shortcut)**.

## 3. Find your save

Master League saves are in:

```
%USERPROFILE%\Documents\KONAMI\eFootball PES 2021 SEASON UPDATE\<your ID>\
```

They are named `ML0000000x` (one file per Master League slot). Paste the path above into
the File Explorer address bar to open the folder.

> **Steam Cloud:** if Steam Cloud is enabled for PES 2021, Steam may replace an edited save
> with the copy stored online. If your changes disappear, disable cloud sync for the game
> (*Steam → Library → right-click PES 2021 → Properties → General → Steam Cloud*), edit
> again, and re-enable it after the game has saved once.

## 4. Edit

1. **Close PES 2021.**
2. In the editor click **Open save…** (or drag the `ML` file onto the window).
3. Change the values and click **Save**. A backup named
   `ML0000000x.backup-YYYYMMDD-HHMMSS` is created next to the save first.

See [USAGE.md](USAGE.md) for every field and option.

## Command line (optional)

`command-line\pes21-ml.exe` is the terminal version (double-clicking it only shows help).
In PowerShell, from the extracted folder:

```powershell
.\command-line\pes21-ml.exe show "$env:USERPROFILE\Documents\KONAMI\eFootball PES 2021 SEASON UPDATE\<your ID>\ML00000001"
.\command-line\pes21-ml.exe set  "...\ML00000001" --max-money
```

## Uninstall

Delete the folder you extracted. The editor keeps no settings anywhere else. Backups of
your saves stay next to the saves until you delete them.

## Troubleshooting

| Problem | Fix |
|---|---|
| The window does not open / OpenGL error | Update the graphics driver. On virtual machines, enable 3D acceleration. |
| "the save changed on disk while editing" | The game (or Steam Cloud) wrote the file. Close the game and try again. |
| "Access is denied" when saving | The save is read-only or open in another program. Close the game and check the file's properties. |
| Antivirus warning | Unsigned, newly released programs are often flagged by reputation. Check the SHA-256 hash or build from source (see the README). |
