//! Compiles the built-in icon themes in (`themes/icon-themes`, the
//! kanso-themes submodule's: `icon_themes::BUILTIN_FILES`; the color themes
//! are compiled into kanso-engine, `kanso_engine::theme::builtin::BUILTIN`). On Windows, also puts the app icon
//! (assets/icon/kanso.ico) into kanso.exe so Explorer, the taskbar and the
//! window show it, and keeps DirectML from loading with the app.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    builtin_icon_themes();
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        // ONNX Runtime, linked in for voice input (kanso-voice), carries
        // DirectML's setup code, which makes the exe import DirectML and
        // Direct3D 12. Voice input runs on the CPU and never calls them, so
        // they load only if called: never, rather than at every launch.
        for dll in ["DirectML.dll", "d3d12.dll"] {
            println!("cargo:rustc-link-arg-bins=/DELAYLOAD:{dll}");
        }
        println!("cargo:rustc-link-arg-bins=delayimp.lib");
    }
    #[cfg(windows)]
    if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows") {
        windows_icon();
    }
}

#[cfg(windows)]
fn windows_icon() {
    use std::path::PathBuf;

    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let icon = manifest_dir.join("../../assets/icon/kanso.ico");
    println!("cargo:rerun-if-changed={}", icon.display());
    let icon = icon.canonicalize().expect("assets/icon/kanso.ico exists");
    // rc.exe wants the path as a C string, without the `\\?\` prefix.
    let icon = icon.to_string_lossy();
    let icon = icon.trim_start_matches(r"\\?\").replace('\\', "\\\\");
    let out_dir = PathBuf::from(std::env::var_os("OUT_DIR").unwrap());
    let rc = out_dir.join("kanso.rc");
    std::fs::write(&rc, format!("1 ICON \"{icon}\"\n")).expect("writing kanso.rc");
    embed_resource::compile(&rc, embed_resource::NONE)
        .manifest_optional()
        .expect("compiling the icon resource");
}

/// Writes `builtin_icon_themes.rs` into `OUT_DIR`: for each folder of
/// `themes/icon-themes/` holding a theme, its ID, its JSON and every SVG under it,
/// by path, as `include_bytes!`s. Only `.json` and `.svg` files go in.
fn builtin_icon_themes() {
    use std::fmt::Write as _;
    use std::path::{Path, PathBuf};

    fn svgs(dir: &Path, prefix: &str, out: &mut Vec<(String, PathBuf)>) {
        let Ok(read) = std::fs::read_dir(dir) else {
            return;
        };
        let mut entries: Vec<PathBuf> = read.filter_map(Result::ok).map(|e| e.path()).collect();
        entries.sort();
        for path in entries {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            if path.is_dir() {
                svgs(&path, &format!("{prefix}{name}/"), out);
            } else if name.ends_with(".svg") {
                out.push((format!("{prefix}{name}"), path));
            }
        }
    }

    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest_dir.join("../../themes/icon-themes");
    // The themes are the kanso-themes repository, checked out as a
    // submodule; without it Kanso would build with no themes at all.
    assert!(
        root.is_dir(),
        "themes/icon-themes is missing: run `git submodule update --init` (the themes are the kanso-themes submodule)"
    );
    // A directory dependency makes Flutter's hooks runner consider the native
    // assets stale on every test/build. The icon-theme index is the manifest
    // that changes when a theme folder is added.
    let index = root.join("index.json");
    assert!(index.is_file(), "themes/icon-themes/index.json is missing");
    println!("cargo:rerun-if-changed={}", index.display());
    let mut code = String::from(
        "/// The built-in icon themes: (ID, theme JSON, SVGs by path).\n\
         pub const BUILTIN_FILES: &[BuiltinTheme] = &[\n",
    );
    let mut folders: Vec<PathBuf> = std::fs::read_dir(&root)
        .map(|read| read.filter_map(Result::ok).map(|e| e.path()).collect())
        .unwrap_or_default();
    folders.sort();
    for folder in folders.into_iter().filter(|p| p.is_dir()) {
        let id = folder.file_name().unwrap().to_string_lossy().into_owned();
        // The schema's folder isn't a theme.
        if id == "schema" {
            continue;
        }
        let json = std::fs::read_dir(&folder)
            .unwrap()
            .filter_map(Result::ok)
            .map(|e| e.path())
            .find(|p| p.extension().is_some_and(|x| x == "json"));
        let Some(json) = json else {
            continue;
        };
        println!("cargo:rerun-if-changed={}", json.display());
        let mut files = Vec::new();
        svgs(&folder, "", &mut files);
        writeln!(
            code,
            "    ({id:?}, include_str!({:?}), &[",
            json.display().to_string()
        )
        .unwrap();
        for (rel, path) in files {
            writeln!(
                code,
                "        ({rel:?}, include_bytes!({:?})),",
                path.display().to_string()
            )
            .unwrap();
        }
        code.push_str("    ]),\n");
    }
    code.push_str("];\n");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("builtin_icon_themes.rs");
    std::fs::write(out, code).expect("writing builtin_icon_themes.rs");
}
