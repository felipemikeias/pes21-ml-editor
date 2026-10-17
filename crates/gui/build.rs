//! Embeds the icon, version information and application manifest into the Windows
//! executable. Does nothing for other targets.
//!
//! Release builds set `PES21_REQUIRE_WINDOWS_RESOURCES=1` so a missing resource
//! compiler fails the build instead of silently producing an exe without an icon.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Quotes a path for an `.rc` string literal.
fn rc_path(p: &Path) -> String {
    p.display()
        .to_string()
        .replace('\\', "\\\\")
        .replace('"', "\"\"")
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=../../assets/icon.ico");
    println!("cargo:rerun-if-changed=windows/pes21-ml-editor.manifest");
    println!("cargo:rerun-if-env-changed=PES21_REQUIRE_WINDOWS_RESOURCES");
    if env::var("CARGO_CFG_TARGET_OS").as_deref() != Ok("windows") {
        return;
    }

    let crate_dir = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let icon = crate_dir.join("../../assets/icon.ico");
    let manifest = crate_dir.join("windows/pes21-ml-editor.manifest");
    let version = env::var("CARGO_PKG_VERSION").expect("CARGO_PKG_VERSION");
    let part = |name: &str| env::var(name).unwrap_or_else(|_| "0".into());
    let numeric = format!(
        "{},{},{},0",
        part("CARGO_PKG_VERSION_MAJOR"),
        part("CARGO_PKG_VERSION_MINOR"),
        part("CARGO_PKG_VERSION_PATCH")
    );

    let rc = format!(
        r#"1 ICON "{icon}"
1 24 "{manifest}"
1 VERSIONINFO
FILEVERSION {numeric}
PRODUCTVERSION {numeric}
FILEFLAGSMASK 0x3F
FILEFLAGS 0x0
FILEOS 0x40004
FILETYPE 0x1
FILESUBTYPE 0x0
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "CompanyName", "pes21-ml-editor contributors"
      VALUE "FileDescription", "PES 2021 ML Editor"
      VALUE "FileVersion", "{version}"
      VALUE "InternalName", "pes21-ml-editor"
      VALUE "LegalCopyright", "MIT OR Apache-2.0"
      VALUE "OriginalFilename", "pes21-ml-editor.exe"
      VALUE "ProductName", "PES 2021 ML Editor"
      VALUE "ProductVersion", "{version}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#,
        icon = rc_path(&icon),
        manifest = rc_path(&manifest),
    );
    let rc_file = out_dir.join("pes21-ml-editor.rc");
    fs::write(&rc_file, rc).expect("write .rc");

    let result = embed_resource::compile(&rc_file, embed_resource::NONE).manifest_required();
    if let Err(e) = result {
        if env::var("PES21_REQUIRE_WINDOWS_RESOURCES").as_deref() == Ok("1") {
            panic!("could not embed Windows resources: {e}");
        }
        println!("cargo:warning=Windows resources (icon, version info) not embedded: {e}");
    }
}
