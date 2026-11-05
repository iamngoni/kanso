//! What the worker tests share: where the worker binary and the PDFium
//! library are (`scripts/fetch-pdfium.sh` puts the library in the source tree).

#![allow(dead_code)]

use std::path::{Path, PathBuf};

use kanso_pdf::client::WorkerPaths;

pub const WORKER: &str = env!("CARGO_BIN_EXE_kanso-pdf-worker");

pub fn library() -> PathBuf {
    let platform = match (std::env::consts::OS, std::env::consts::ARCH) {
        ("macos", "aarch64") => "mac-arm64",
        ("macos", _) => "mac-x64",
        ("linux", "aarch64") => "linux-arm64",
        ("linux", _) => "linux-x64",
        ("windows", "aarch64") => "win-arm64",
        _ => "win-x64",
    };
    let (sub, name) = if cfg!(target_os = "macos") {
        ("lib", "libpdfium.dylib")
    } else if cfg!(windows) {
        ("bin", "pdfium.dll")
    } else {
        ("lib", "libpdfium.so")
    };
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../vendor/pdfium")
        .join(platform)
        .join(sub)
        .join(name);
    assert!(
        path.is_file(),
        "PDFium is missing: run scripts/fetch-pdfium.sh ({path:?})"
    );
    path.canonicalize().unwrap()
}

pub fn paths() -> WorkerPaths {
    WorkerPaths {
        exe: WORKER.into(),
        library: Some(library()),
    }
}
