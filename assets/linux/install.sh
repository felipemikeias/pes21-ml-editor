#!/bin/sh
# Installs PES 2021 ML Editor for the current user (no root needed):
#   ~/.local/bin/pes21-ml-editor, ~/.local/bin/pes21-ml,
#   an applications-menu entry and the icon used by docks and task switchers.
# Run with --uninstall to remove exactly those files.
set -eu

here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
data="${XDG_DATA_HOME:-$HOME/.local/share}"
bin="$HOME/.local/bin"
app="$data/applications/pes21-ml-editor.desktop"
icon="$data/icons/hicolor/256x256/apps/pes21-ml-editor.png"

refresh() {
    update-desktop-database "$data/applications" 2>/dev/null || true
    gtk-update-icon-cache -q -t "$data/icons/hicolor" 2>/dev/null || true
}

if [ "${1:-}" = "--uninstall" ]; then
    rm -f "$bin/pes21-ml-editor" "$bin/pes21-ml" "$app" "$icon"
    refresh
    echo "PES 2021 ML Editor removed."
    exit 0
fi

mkdir -p "$bin" "$(dirname "$app")" "$(dirname "$icon")"
install -m 755 "$here/pes21-ml-editor" "$bin/pes21-ml-editor"
install -m 755 "$here/command-line/pes21-ml" "$bin/pes21-ml"
install -m 644 "$here/pes21-ml-editor.png" "$icon"
# Absolute Exec path: ~/.local/bin is not always on the PATH of desktop launchers.
sed "s|^Exec=pes21-ml-editor|Exec=$bin/pes21-ml-editor|" "$here/pes21-ml-editor.desktop" > "$app"
chmod 644 "$app"
refresh
echo "PES 2021 ML Editor installed. Open it from the applications menu."
