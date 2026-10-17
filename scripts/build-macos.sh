#!/bin/sh
# Builds "PES 2021 ML Editor.app" and a .dmg on a Mac, the same way the release workflow
# does, so the macOS version can be made and tested locally.
#
#   scripts/build-macos.sh                   # this Mac's architecture -> dist/
#   scripts/build-macos.sh --arch universal  # Apple Silicon + Intel in one app (~2x size)
#   scripts/build-macos.sh --no-dmg          # only the .app
#
# Needs Rust (https://rustup.rs) and the Xcode command-line tools (xcode-select --install).
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
arch=native
make_dmg=1
out="$root/dist"

while [ $# -gt 0 ]; do
    case "$1" in
        --arch) arch="${2:?--arch needs native, aarch64, x86_64 or universal}"; shift ;;
        --no-dmg) make_dmg=0 ;;
        --out) out="${2:?--out needs a folder}"; shift ;;
        -h|--help) sed -n '2,9p' "$0" | sed 's/^# \{0,1\}//'; exit 0 ;;
        *) echo "unknown option: $1" >&2; exit 2 ;;
    esac
    shift
done

[ "$(uname -s)" = Darwin ] || { echo "error: run this on macOS" >&2; exit 1; }
command -v cargo >/dev/null || { echo "error: cargo not found; install Rust from https://rustup.rs" >&2; exit 1; }

if [ "$arch" = native ]; then
    case "$(uname -m)" in arm64) arch=aarch64 ;; *) arch=x86_64 ;; esac
fi
case "$arch" in
    aarch64|x86_64) targets="$arch-apple-darwin" ;;
    universal) targets="aarch64-apple-darwin x86_64-apple-darwin" ;;
    *) echo "error: --arch must be native, aarch64, x86_64 or universal" >&2; exit 2 ;;
esac

cd "$root"
version=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)
pkg="pes21-ml-editor-$version-macos-$arch"

# Matches LSMinimumSystemVersion in assets/macos/Info.plist.
export MACOSX_DEPLOYMENT_TARGET=11.0

for t in $targets; do
    if command -v rustup >/dev/null && ! rustup target list --installed | grep -qx "$t"; then
        echo "error: missing Rust target $t; install it with: rustup target add $t" >&2
        exit 1
    fi
    cargo build --workspace --locked --release --target "$t"
done

stage=$(mktemp -d)
trap 'rm -rf "$stage"' EXIT INT TERM
mkdir -p "$stage/$pkg/command-line"
app="$stage/$pkg/PES 2021 ML Editor.app"
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"

# One binary per architecture, or a fat binary for --arch universal.
for bin in pes21-ml-editor pes21-ml; do
    case "$bin" in pes21-ml-editor) dest="$app/Contents/MacOS/$bin" ;; *) dest="$stage/$pkg/command-line/$bin" ;; esac
    set --
    for t in $targets; do set -- "$@" "target/$t/release/$bin"; done
    if [ $# -gt 1 ]; then lipo -create "$@" -output "$dest"; else cp "$1" "$dest"; fi
done

cp assets/icon.icns "$app/Contents/Resources/AppIcon.icns"
sed "s/@VERSION@/$version/g" assets/macos/Info.plist > "$app/Contents/Info.plist"
plutil -lint -s "$app/Contents/Info.plist"
# Ad-hoc signature: required to run on Apple Silicon (not a Developer ID).
codesign --force --sign - "$app"
codesign --force --sign - "$stage/$pkg/command-line/pes21-ml"
codesign --verify --strict "$app"

cp README.md LICENSE-MIT LICENSE-APACHE THIRD_PARTY_NOTICES.md \
    docs/INSTALL-MACOS.md docs/USAGE.md "$stage/$pkg/"

mkdir -p "$out"
rm -rf "$out/PES 2021 ML Editor.app"
cp -R "$app" "$out/"
echo "app: $out/PES 2021 ML Editor.app ($(du -sh "$app" | cut -f1))"

if [ "$make_dmg" = 1 ]; then
    ln -s /Applications "$stage/$pkg/Applications"
    # ULFO (LZFSE) is smaller than zlib and opens on every macOS the app supports.
    hdiutil create -quiet -volname "PES 2021 ML Editor" -srcfolder "$stage/$pkg" \
        -ov -format ULFO "$out/$pkg.dmg"
    (cd "$out" && shasum -a 256 "$pkg.dmg" > "$pkg.dmg.sha256")
    echo "dmg: $out/$pkg.dmg ($(du -h "$out/$pkg.dmg" | cut -f1))"
fi
