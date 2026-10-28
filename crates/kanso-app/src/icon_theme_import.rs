//! The desktop command for importing icon themes; conversion is shared with the phone.

use std::path::{Path, PathBuf};

use anyhow::{Context as _, bail};

use crate::icon_theme_file;
use crate::icon_themes::{builtins, icon_themes_dir};

use kanso_engine::icons::import::Imports;

const HELP: &str = "\
Usage: kanso icon-theme <command>

Commands:
  import <path>... [--out <dir>]
                    Convert icon themes into Kanso icon theme folders: a Zed
                    icon theme (an extension's icon_themes/*.json, its
                    folder or .tar.gz), a theme package carrying one (its
                    package folder or .tgz) or a Kanso icon theme folder
                    (checked and copied). They go in --out, or Kanso's icon
                    themes folder
  check <folder>... Read Kanso icon theme folders and report any problem
  schema [--out <file>]
                    Print the icon theme JSON Schema, or write it to <file>
  help              Show this message";

/// Converts an icon theme source into the library's icon-theme folder.
pub fn import_path(source: &Path, dir: &Path) -> Result<Imports, String> {
    let built_in = builtins();
    let known_icon = crate::assets::is_app_icon;
    let is_builtin_name = |name: &str| {
        built_in
            .iter()
            .any(|theme| theme.name.eq_ignore_ascii_case(name))
    };
    let is_builtin_id = |id: &str| built_in.iter().any(|theme| theme.id == id);
    kanso_engine::icons::import::import_path(
        source,
        dir,
        &known_icon,
        &is_builtin_name,
        &is_builtin_id,
    )
}

pub fn run_cli(data_dir: &Path, args: &[String]) -> anyhow::Result<()> {
    let Some((command, rest)) = args.split_first() else {
        println!("{HELP}");
        return Ok(());
    };
    let (paths, out) = split_out(rest)?;
    match command.as_str() {
        "import" => {
            if paths.is_empty() {
                bail!("`kanso icon-theme import` needs an icon theme to import\n\n{HELP}");
            }
            let dir = out.unwrap_or_else(|| icon_themes_dir(data_dir));
            let mut failed = 0;
            for path in &paths {
                match import_path(path, &dir) {
                    Ok(imports) => {
                        for report in &imports.reports {
                            println!("{}", report.details());
                        }
                        for skipped in &imports.skipped {
                            eprintln!("{}: {skipped}", path.display());
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
                let mut notices = Vec::new();
                let theme =
                    icon_theme_file::read_folder(path, crate::origin::Origin::Own, &mut notices);
                match theme {
                    Some(theme) if notices.is_empty() => println!(
                        "{}: reads well ({}, {} icons)",
                        path.display(),
                        theme.name,
                        theme.count()
                    ),
                    _ => {
                        bad += 1;
                        for notice in &notices {
                            println!("{}: {}", path.display(), notice.sentence());
                        }
                        if notices.is_empty() {
                            println!("{}: isn't an icon theme folder", path.display());
                        }
                    }
                }
            }
            if bad > 0 {
                bail!("{bad} of {} icon themes have problems", paths.len());
            }
            Ok(())
        }
        "schema" => {
            let mut text =
                serde_json::to_string_pretty(&icon_theme_file::schema()).unwrap_or_default();
            text.push('\n');
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
        other => bail!("unknown icon-theme command `{other}`\n\n{HELP}"),
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

#[cfg(test)]
mod tests {
    use super::run_cli;

    #[test]
    fn the_icon_theme_cli_imports_a_kanso_theme_through_the_engine() {
        let root = tempfile::tempdir().unwrap();
        let source = root.path().join("sketch");
        std::fs::create_dir_all(source.join("icons")).unwrap();
        std::fs::write(
            source.join("sketch.json"),
            r#"{"schema_version":1,"name":"Sketch","author":"Ada","license":"MIT","icons":{"search":"icons/search.svg"}}"#,
        )
        .unwrap();
        std::fs::write(
            source.join("icons/search.svg"),
            r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"><path d="M2 2h20v20H2z"/></svg>"#,
        )
        .unwrap();
        let out = root.path().join("icon-themes");
        let args = vec![
            "import".to_owned(),
            source.display().to_string(),
            "--out".to_owned(),
            out.display().to_string(),
        ];

        run_cli(root.path(), &args).unwrap();

        let imported = out.join("sketch");
        assert!(imported.join("sketch.json").is_file());
        assert!(imported.join("icons/search.svg").is_file());
    }
}
