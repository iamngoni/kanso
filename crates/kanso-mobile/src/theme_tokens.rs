//! A theme file fitted for drawing, as plain rows the shell reads.

use kanso_engine::theme::file::ThemeFile;
use kanso_engine::theme::{Colors, Tokens};

use crate::errors::KansoError;
use crate::model::{ColorRow, PaletteDto, SyntaxStyleRow, ThemeTokens};

/// Reads `theme_json`, a theme file, and fits it as one theme for the
/// interface, the editor's syntax and the preview.
pub fn fit(theme_json: &str) -> Result<ThemeTokens, KansoError> {
    let file = ThemeFile::parse(theme_json)
        .map_err(|err| KansoError::invalid(format!("The theme {err}.")))?;
    Ok(fit_file(&file).0)
}

/// A parsed theme fitted as one theme for the interface, the editor's syntax
/// and the preview: the rows the shell reads, and the fitted syntax styles
/// themselves.
pub fn fit_file(file: &ThemeFile) -> (ThemeTokens, Tokens) {
    let resolved = file.resolve();
    let tokens = Tokens::fit(&resolved, &resolved, &resolved);
    (rows(&tokens), tokens)
}

pub(crate) fn rows(tokens: &Tokens) -> ThemeTokens {
    ThemeTokens {
        palette: palette(tokens),
        ui: ui_colors(&tokens.ui),
        caret: tokens.caret.0,
        selection: tokens.selection.rgb.0,
        selection_alpha: tokens.selection.alpha,
        syntax: tokens
            .syntax
            .iter()
            .map(|(token, style)| SyntaxStyleRow {
                token: (*token).to_owned(),
                color: style.color.map(|color| color.0),
                italic: style.italic,
                weight: style.weight,
            })
            .collect(),
        name: tokens.name.clone(),
        dark: tokens.dark,
    }
}

/// The colors the preview and drawings are made in.
fn palette(tokens: &Tokens) -> PaletteDto {
    let preview = &tokens.preview;
    PaletteDto {
        dark: tokens.dark,
        background: tokens.ui.editor.0,
        text: preview.text.0,
        muted: tokens.ui.text_muted.0,
        accent: tokens.ui.link.0,
        border: preview.border.0,
        surface: preview.code_background.0,
        heading: preview.heading.map(|color| color.0),
        quote: preview.quote.0,
        task_done: preview.task_done.0,
        code_text: preview.code_text.map(|color| color.0),
        inline_code_background: preview.inline_code_background.0,
        inline_code_text: preview.inline_code_text.map(|color| color.0),
        table_header_background: preview.table_header_background.map(|color| color.0),
    }
}

fn ui_colors(colors: &Colors) -> Vec<ColorRow> {
    [
        ("app", colors.app),
        ("sidebar", colors.sidebar),
        ("list", colors.list),
        ("editor", colors.editor),
        ("elevated", colors.elevated),
        ("fill", colors.fill),
        ("text", colors.text),
        ("text_secondary", colors.text_secondary),
        ("text_muted", colors.text_muted),
        ("divider", colors.divider),
        ("accent", colors.accent),
        ("link", colors.link),
        ("row_active", colors.row_active),
        ("row_hover", colors.row_hover),
        ("sidebar_active", colors.sidebar_active),
        ("sidebar_hover", colors.sidebar_hover),
        ("hover", colors.hover),
        ("selected", colors.selected),
        ("active_line", colors.active_line),
        ("button", colors.button),
        ("button_text", colors.button_text),
        ("code", colors.code),
        ("code_text", colors.code_text),
        ("syntax_keyword", colors.syntax_keyword),
        ("syntax_string", colors.syntax_string),
        ("success", colors.success),
        ("warning", colors.warning),
        ("danger", colors.danger),
    ]
    .into_iter()
    .map(|(name, color)| ColorRow {
        name: name.to_owned(),
        rgb: color.0,
    })
    .collect()
}
