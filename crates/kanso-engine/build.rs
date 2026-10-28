//! Compiles the built-in color themes in (`themes/`, the kanso-themes
//! submodule's): `kanso_engine::theme::builtin::BUILTIN`, the one list the
//! desktop's Themes page and the phones' theme picker both read.

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    builtin_themes();
}

/// The order the Themes page had before this table was generated: Kanso's
/// own first, then each family's light theme beside its dark one. New themes
/// follow in `index.json`'s order, so adding one only needs a submodule bump.
const KEEP_FIRST: &[&str] = &[
    "kanso-light.json",
    "kanso-dark.json",
    "default-light.json",
    "default-dark.json",
    "github-light.json",
    "github-dark.json",
    "solarized-light.json",
    "solarized-dark.json",
    "gruvbox-light.json",
    "gruvbox-dark.json",
    "catppuccin-latte.json",
    "catppuccin-mocha.json",
    "rose-pine-dawn.json",
    "rose-pine.json",
    "kanagawa-lotus.json",
    "kanagawa-wave.json",
    "kanagawa-dragon.json",
    "nord.json",
    "dracula.json",
    "tokyo-night.json",
];

/// Writes `builtin_themes.rs` into `OUT_DIR`: the color themes in
/// `themes/index.json`, by file name, as `include_str!`s. The first two are
/// Kanso's own light and dark themes, which the defaults fall back to.
fn builtin_themes() {
    use std::collections::BTreeSet;
    use std::fmt::Write as _;
    use std::path::PathBuf;

    let manifest_dir = PathBuf::from(std::env::var_os("CARGO_MANIFEST_DIR").unwrap());
    let root = manifest_dir.join("../../themes");
    // The themes are the kanso-themes repository, checked out as a
    // submodule; without it Kanso would build with no themes at all.
    assert!(
        root.is_dir(),
        "themes is missing: run `git submodule update --init` (the themes are the kanso-themes submodule)"
    );
    // Watch the index and every file it names, never the directory itself:
    // directory watches make Flutter's hooks runner report a changed file on
    // every build and rebuild the native library for no reason.
    let index = root.join("index.json");
    println!("cargo:rerun-if-changed={}", index.display());
    let text = std::fs::read_to_string(&index)
        .unwrap_or_else(|err| panic!("{} couldn't be read: {err}", index.display()));
    let value: serde_json::Value = serde_json::from_str(&text)
        .unwrap_or_else(|err| panic!("{} isn't valid JSON: {err}", index.display()));
    let listed: Vec<&str> = value
        .get("themes")
        .and_then(serde_json::Value::as_array)
        .unwrap_or_else(|| panic!("{} has no themes list", index.display()))
        .iter()
        .map(|theme| {
            theme
                .get("file")
                .and_then(serde_json::Value::as_str)
                .unwrap_or_else(|| panic!("{} has a theme with no file", index.display()))
        })
        .collect();

    // The previous order for the themes that had one, then the rest as the
    // index lists them.
    let known: BTreeSet<&str> = listed.iter().copied().collect();
    assert_eq!(
        known.len(),
        listed.len(),
        "{} lists a theme twice",
        index.display()
    );
    let mut ordered: Vec<&str> = KEEP_FIRST
        .iter()
        .copied()
        .filter(|file| known.contains(file))
        .collect();
    ordered.extend(
        listed
            .iter()
            .copied()
            .filter(|file| !KEEP_FIRST.contains(file)),
    );
    assert_eq!(
        ordered.len(),
        listed.len(),
        "{} lists a theme the table doesn't",
        index.display()
    );
    assert_eq!(
        ordered.first().copied(),
        Some("kanso-light.json"),
        "BUILTIN[0] must be Kanso's light theme"
    );
    assert_eq!(
        ordered.get(1).copied(),
        Some("kanso-dark.json"),
        "BUILTIN[1] must be Kanso's dark theme"
    );

    let mut code = String::from(
        "/// The built-in themes' files, in the order the Themes page shows them:\n\
         /// Kanso's own first, then each family's light theme beside its dark one.\n\
         pub const BUILTIN: &[(&str, &str)] = &[\n",
    );
    for file in ordered {
        let path = root.join(file);
        println!("cargo:rerun-if-changed={}", path.display());
        writeln!(
            code,
            "    ({file:?}, include_str!({:?})),",
            path.display().to_string()
        )
        .unwrap();
    }
    code.push_str("];\n");
    let out = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("builtin_themes.rs");
    std::fs::write(out, code).expect("writing builtin_themes.rs");
}
