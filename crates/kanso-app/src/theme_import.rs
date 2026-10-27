//! The `kanso theme` command. The importing, the checks and the theme file
//! format are `kanso_engine::theme` (`import`, `file`), so the phone calls the
//! same code.
//!
//! The same checks back `kanso theme check`, so a theme written by hand is
//! held to what the built-in ones are.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};
use serde_json::Value;

use crate::themes::themes_dir;

pub use kanso_engine::theme::file::problems;
use kanso_engine::theme::import::{import_into as import_files, index, read_theme};
pub use kanso_engine::theme::import::{import_path, slug, strip_jsonc};

const HELP: &str = "\
Usage: kanso theme <command>

Commands:
  import <path>... [--out <dir>]
                    Convert themes into Kanso theme files: a theme package
                    (its package folder or .tgz), a Zed theme family (.json)
                    or a Kanso theme (.json, checked and copied). They go in
                    --out, or Kanso's themes folder.
  check <file>...   Read Kanso theme files and report any problem: an error
                    in the file, or a color that doesn't read where it is
                    drawn
  fmt <file>...     Rewrite Kanso theme files in the canonical form
  index <dir> [--out <file>]
                    Write an index of the theme files in <dir>
                    (default <dir>/index.json)
  schema [--out <file>]
                    Print the theme file JSON Schema, or write it to <file>
  help              Show this message";

/// `kanso theme …`.
// no-prop-test: argument dispatch; `the_cli_imports_checks_formats_and_indexes`
// runs every command.
pub fn run_cli(data_dir: &Path, args: &[String]) -> anyhow::Result<()> {
    let Some((command, rest)) = args.split_first() else {
        println!("{HELP}");
        return Ok(());
    };
    let (paths, out) = split_out(rest)?;
    match command.as_str() {
        "import" => {
            if paths.is_empty() {
                bail!("`kanso theme import` needs a theme to import\n\n{HELP}");
            }
            let dir = out.unwrap_or_else(|| themes_dir(data_dir));
            let mut failed = 0;
            for path in &paths {
                match import_files(path, &dir) {
                    Ok(written) => {
                        for w in written {
                            println!("Wrote {} ({})", w.path.display(), w.name);
                        }
                    }
                    Err(err) => {
                        failed += 1;
                        eprintln!("{}: {err}", path.display());
                    }
                }
            }
            if failed > 0 {
                bail!("{failed} of {} couldn't be imported", paths.len());
            }
            Ok(())
        }
        "check" => {
            let mut bad = 0;
            for path in &paths {
                let problems = match read_theme(path) {
                    Ok(theme) => problems(&theme),
                    Err(err) => vec![err],
                };
                if problems.is_empty() {
                    println!("{}: reads well", path.display());
                } else {
                    bad += 1;
                    for problem in problems {
                        println!("{}: {problem}", path.display());
                    }
                }
            }
            if bad > 0 {
                bail!("{bad} of {} theme files have problems", paths.len());
            }
            Ok(())
        }
        "fmt" => {
            for path in &paths {
                let theme =
                    read_theme(path).map_err(|err| anyhow::anyhow!("{}: {err}", path.display()))?;
                let text = theme.to_json_string();
                if std::fs::read_to_string(path).ok().as_deref() == Some(text.as_str()) {
                    println!("{} is already formatted", path.display());
                } else {
                    std::fs::write(path, text)
                        .with_context(|| format!("writing {}", path.display()))?;
                    println!("Formatted {}", path.display());
                }
            }
            Ok(())
        }
        "index" => {
            let [dir] = paths.as_slice() else {
                bail!("`kanso theme index` takes one folder\n\n{HELP}");
            };
            let index = index(dir).map_err(anyhow::Error::msg)?;
            let out = out.unwrap_or_else(|| dir.join("index.json"));
            std::fs::write(&out, pretty(&index))
                .with_context(|| format!("writing {}", out.display()))?;
            println!("Wrote {}", out.display());
            Ok(())
        }
        "schema" => {
            let text = pretty(&crate::theme_file::schema());
            match out {
                Some(out) => {
                    std::fs::write(&out, text)
                        .with_context(|| format!("writing {}", out.display()))?;
                    println!("Wrote {}", out.display());
                }
                None => print!("{text}"),
            }
            Ok(())
        }
        "help" | "-h" | "--help" => {
            println!("{HELP}");
            Ok(())
        }
        other => bail!("unknown theme command `{other}`\n\n{HELP}"),
    }
}

/// The arguments, and `--out`'s value apart from them.
fn split_out(args: &[String]) -> anyhow::Result<(Vec<PathBuf>, Option<PathBuf>)> {
    let mut paths = Vec::new();
    let mut out = None;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--out" | "-o" => out = Some(PathBuf::from(args.next().context("--out needs a path")?)),
            other if other.starts_with("--") => bail!("unexpected option `{other}`\n\n{HELP}"),
            other => paths.push(PathBuf::from(other)),
        }
    }
    Ok((paths, out))
}

fn pretty(value: &Value) -> String {
    let mut text = serde_json::to_string_pretty(value).unwrap_or_default();
    text.push('\n');
    text
}

#[cfg(test)]
mod tests {
    use super::problems;
    use super::run_cli;
    use crate::theme_file::ThemeFile;

    /// The CLI end to end on a small theme: import, check, fmt, index and
    /// schema.
    #[test]
    fn the_cli_imports_checks_formats_and_indexes() {
        let dir = tempfile::tempdir().unwrap();
        let pkg = dir.path().join("pkg");
        std::fs::create_dir_all(pkg.join("styles")).unwrap();
        std::fs::write(
            pkg.join("package.json"),
            r#"{"name": "grey-dark", "theme": true, "themeAppearance": "dark", "styleSheets": ["ui.css"], "author": "A <a@b.c>", "license": "MIT"}"#,
        )
        .unwrap();
        std::fs::write(
            pkg.join("styles/ui.css"),
            ":root { --editor-background: #222; --text-color: #555; --primary-color: #333; }",
        )
        .unwrap();
        let out = dir.path().join("out");
        let args = |a: &[&str]| a.iter().map(|s| (*s).to_owned()).collect::<Vec<_>>();
        let data = dir.path().join("data");
        run_cli(
            &data,
            &args(&[
                "import",
                pkg.to_str().unwrap(),
                "--out",
                out.to_str().unwrap(),
            ]),
        )
        .unwrap();
        let file = out.join("grey-dark.json");
        let theme = ThemeFile::parse(&std::fs::read_to_string(&file).unwrap()).unwrap();
        assert_eq!(theme.author, "A");
        assert!(problems(&theme).is_empty(), "{:?}", problems(&theme));
        run_cli(&data, &args(&["check", file.to_str().unwrap()])).unwrap();
        assert!(
            !data.exists(),
            "an import with --out leaves the data folder alone"
        );
        // A theme that doesn't read fails the check.
        let unreadable = dir.path().join("unreadable.json");
        std::fs::write(
            &unreadable,
            r##"{"name": "U", "appearance": "dark", "ui": {"editor": "#222222", "text": "#333333", "accent": "#444444"}}"##,
        )
        .unwrap();
        assert!(run_cli(&data, &args(&["check", unreadable.to_str().unwrap()])).is_err());
        // fmt rewrites in the canonical form.
        run_cli(&data, &args(&["fmt", unreadable.to_str().unwrap()])).unwrap();
        let text = std::fs::read_to_string(&unreadable).unwrap();
        assert_eq!(ThemeFile::parse(&text).unwrap().to_json_string(), text);
        // index lists every theme in the folder but itself.
        run_cli(&data, &args(&["index", out.to_str().unwrap()])).unwrap();
        run_cli(&data, &args(&["index", out.to_str().unwrap()])).unwrap();
        let index: serde_json::Value =
            serde_json::from_str(&std::fs::read_to_string(out.join("index.json")).unwrap())
                .unwrap();
        assert_eq!(index["schema_version"], 1);
        assert_eq!(index["themes"].as_array().unwrap().len(), 1);
        assert_eq!(index["themes"][0]["file"], "grey-dark.json");
        assert_eq!(index["themes"][0]["name"], "Grey Dark");
        assert_eq!(index["themes"][0]["colors"]["editor"], "#222222");
        let schema = dir.path().join("schema.json");
        run_cli(&data, &args(&["schema", "--out", schema.to_str().unwrap()])).unwrap();
        let text = std::fs::read_to_string(&schema).unwrap();
        assert!(text.ends_with("}\n"));
        assert_eq!(
            serde_json::from_str::<serde_json::Value>(&text).unwrap(),
            crate::theme_file::schema()
        );
        assert!(run_cli(&data, &args(&["nonsense"])).is_err());
        assert!(run_cli(&data, &args(&["help"])).is_ok());
    }
}
