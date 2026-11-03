//! Apple Notes' HTML and text as Markdown: what the share sheet hands over
//! (a note's HTML, or its text) and what Notes exports as `.html`.
//!
//! The one place the rules live, so the phone's share extension and the
//! Files route read a note the same way: checklists become `- [ ]` and
//! `- [x]` tasks, a note that is only a drawing or a scan says so, and the
//! note's own title is not repeated in its body. Pure.

use htmd::options::{BrStyle, BulletListMarker, HrStyle, Options};

use super::html::{MAX_DEPTH, nesting_depth};

/// Stand-ins for an open and a done checkbox while `htmd` runs, which would
/// escape `[ ]` in text. Private-use characters, removed from the input.
const OPEN: char = '\u{E000}';
const DONE: char = '\u{E001}';

/// The body of a note that holds a drawing and no text.
pub const DRAWING: &str = "(drawing)";
/// The body of a note that holds a scan and no text.
pub const SCAN: &str = "(scan)";

/// Glyphs Notes and the system use for an unchecked box, and a checked one.
const OPEN_GLYPHS: [char; 6] = ['☐', '◻', '⬜', '◯', '○', '□'];
const DONE_GLYPHS: [char; 5] = ['☑', '✅', '✔', '✓', '☒'];

/// `html` as Markdown with checklists as tasks. `None` when nothing is left
/// or the page nests too deeply to convert.
pub fn html_to_markdown(html: &str) -> Option<String> {
    if nesting_depth(html) > MAX_DEPTH {
        return None;
    }
    let cleaned: String = html.chars().filter(|c| *c != OPEN && *c != DONE).collect();
    let marked = mark_checklists(&cleaned);
    let converter = htmd::HtmlToMarkdown::builder()
        .options(Options {
            bullet_list_marker: BulletListMarker::Dash,
            hr_style: HrStyle::Dashes,
            br_style: BrStyle::Backslash,
            ul_bullet_spacing: 1,
            ol_number_spacing: 1,
            ..Options::default()
        })
        .skip_tags(vec!["script", "style", "head", "meta", "title", "noscript"])
        .build();
    let markdown = converter.convert(&marked).ok()?;
    let markdown = resolve_marks(&markdown);
    let tidy = tasks_from_glyphs(&tidy(&markdown));
    (!tidy.is_empty()).then_some(tidy)
}

/// Each stand-in mark as `[ ]` or `[x]` and one space, whatever spacing the
/// converter left after it.
fn resolve_marks(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut chars = markdown.chars().peekable();
    while let Some(c) = chars.next() {
        if c == OPEN || c == DONE {
            out.push_str(if c == OPEN { "[ ] " } else { "[x] " });
            while chars.next_if(|n| *n == ' ' || *n == '\u{a0}').is_some() {}
        } else {
            out.push(c);
        }
    }
    out
}

/// A shared note as Markdown: its HTML if there is one, else its text.
///
/// The first line is dropped when it is the `subject` (Notes shares a note
/// under its title, and the body starts with the same line). `None` when the
/// share has no words.
pub fn share_markdown(
    subject: Option<&str>,
    html: Option<&str>,
    text: Option<&str>,
) -> Option<String> {
    let body = html
        .and_then(html_to_markdown)
        .or_else(|| text.map(|t| tasks_from_glyphs(&tidy(t))))
        .filter(|body| !body.trim().is_empty())?;
    Some(without_title(&body, subject))
}

/// `body` without its first line when that line is `title`, ignoring a
/// heading mark, emphasis and case.
pub fn without_title(body: &str, title: Option<&str>) -> String {
    let Some(title) = title.map(str::trim).filter(|t| !t.is_empty()) else {
        return body.to_owned();
    };
    let (first, rest) = body.split_once('\n').unwrap_or((body, ""));
    if same_words(first, title) {
        rest.trim_start_matches('\n').to_owned()
    } else {
        body.to_owned()
    }
}

fn same_words(line: &str, title: &str) -> bool {
    let plain = |s: &str| -> String {
        s.trim()
            .trim_start_matches('#')
            .chars()
            .filter(|c| !matches!(c, '*' | '_' | '`' | '\\'))
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
            .to_lowercase()
    };
    let (line, title) = (plain(line), plain(title));
    !line.is_empty() && line == title
}

/// `body` with `(drawing)` first when all it holds is pictures, so a note of
/// only a drawing or a scan still says what it is.
pub fn say_picture_only(body: &str) -> String {
    let mut words = String::new();
    let mut rest = body;
    while let Some(at) = rest.find("![") {
        words.push_str(&rest[..at]);
        let after = &rest[at..];
        rest = match after
            .find("](")
            .and_then(|m| after[m..].find(')').map(|c| m + c + 1))
        {
            Some(end) => &after[end..],
            None => &after[2..],
        };
    }
    words.push_str(rest);
    if body.contains("![") && words.trim().is_empty() {
        format!("{DRAWING}\n\n{}", body.trim())
    } else {
        body.to_owned()
    }
}

/// What to say for a note with no words but a file: `(scan)` for a PDF,
/// `(drawing)` otherwise.
pub fn placeholder(is_pdf: bool) -> &'static str {
    if is_pdf { SCAN } else { DRAWING }
}

/// Trailing spaces off each line, one blank line at most in a row, none at
/// the ends. Code fences are left as they are.
pub fn tidy(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut blank = 0;
    let mut fenced = false;
    for line in markdown.trim_matches('\n').lines() {
        let line = line.trim_end();
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
        }
        if line.is_empty() && !fenced {
            blank += 1;
            continue;
        }
        if !out.is_empty() {
            out.push_str(if blank > 0 { "\n\n" } else { "\n" });
        }
        blank = 0;
        out.push_str(line);
    }
    out
}

/// Each line that starts with a checkbox glyph (alone or after a list
/// marker) as a task. Lines in code fences are left alone.
pub fn tasks_from_glyphs(text: &str) -> String {
    let mut fenced = false;
    let mut lines = Vec::new();
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            fenced = !fenced;
        }
        lines.push(if fenced {
            line.to_owned()
        } else {
            task_line(line)
        });
    }
    lines.join("\n").trim_end_matches('\n').to_owned()
}

fn task_line(line: &str) -> String {
    let indent: String = line
        .chars()
        .take_while(|c| *c == ' ' || *c == '\t')
        .collect();
    let rest = &line[indent.len()..];
    let rest = rest
        .strip_prefix("- ")
        .or_else(|| rest.strip_prefix("* "))
        .or_else(|| rest.strip_prefix("• "))
        .unwrap_or(rest);
    let mut chars = rest.chars();
    let (Some(first), after) = (chars.next(), chars.as_str()) else {
        return line.to_owned();
    };
    let done = if OPEN_GLYPHS.contains(&first) {
        false
    } else if DONE_GLYPHS.contains(&first) {
        true
    } else {
        return line.to_owned();
    };
    let after = after.trim_start_matches([' ', '\t', '\u{a0}']);
    format!("{indent}- [{}] {after}", if done { 'x' } else { ' ' })
        .trim_end()
        .to_owned()
}

/// `html` with each checklist item given a `OPEN`/`DONE` mark at its start:
/// a `<li>` that says it is checked (`data-checked`, `aria-checked`, a
/// `checked`/`done` class or an unchecked/`todo` one), and each
/// `<input type=checkbox>`, which is replaced by its mark.
fn mark_checklists(html: &str) -> String {
    let mut out = String::with_capacity(html.len());
    let mut cursor = 0;
    // The mark of the open `<li>`, so its own checkbox is not marked twice.
    let mut item_marked = false;
    while let Some(rel) = html[cursor..].find('<') {
        let start = cursor + rel;
        let rest = &html[start + 1..];
        if rest.starts_with("!--") {
            let end = rest.find("-->").map_or(html.len(), |e| start + 1 + e + 3);
            out.push_str(&html[cursor..end]);
            cursor = end;
            continue;
        }
        let Some(gt) = tag_end(rest) else {
            break;
        };
        let end = start + 1 + gt + 1;
        let tag = &html[start..end];
        let lower = tag.to_ascii_lowercase();
        let name: String = lower[1..]
            .trim_start_matches('/')
            .chars()
            .take_while(char::is_ascii_alphanumeric)
            .collect();
        out.push_str(&html[cursor..start]);
        cursor = end;
        let closing = lower.starts_with("</");
        match name.as_str() {
            "li" if !closing => {
                out.push_str(tag);
                item_marked = match list_item_state(&lower) {
                    Some(true) => {
                        out.push(DONE);
                        true
                    }
                    Some(false) => {
                        out.push(OPEN);
                        true
                    }
                    None => false,
                };
            }
            "li" => {
                item_marked = false;
                out.push_str(tag);
            }
            "input" if !closing && lower.contains("checkbox") => {
                if !item_marked {
                    out.push(if has_attr(&lower, "checked") {
                        DONE
                    } else {
                        OPEN
                    });
                }
            }
            _ => out.push_str(tag),
        }
    }
    out.push_str(&html[cursor..]);
    out
}

/// The offset of the `>` that ends the tag in `rest` (the text after `<`),
/// skipping quoted attribute values.
fn tag_end(rest: &str) -> Option<usize> {
    let mut quote: Option<char> = None;
    for (ix, c) in rest.char_indices() {
        match (quote, c) {
            (Some(q), c) if c == q => quote = None,
            (None, '"' | '\'') => quote = Some(c),
            (None, '>') => return Some(ix),
            _ => {}
        }
    }
    None
}

/// Whether the (lowercased) tag has the attribute `name`, with any value
/// but `false`.
fn has_attr(tag: &str, name: &str) -> bool {
    attr_value(tag, name).is_some_and(|v| v != "false")
}

fn attr_value(tag: &str, name: &str) -> Option<String> {
    let mut from = 0;
    while let Some(rel) = tag[from..].find(name) {
        let at = from + rel;
        from = at + name.len();
        let before_ok =
            tag[..at].ends_with(|c: char| c.is_ascii_whitespace() || c == '"' || c == '\'');
        let after = &tag[from..];
        let after_ok = after.starts_with(['=', ' ', '/', '>']);
        if !before_ok || !after_ok {
            continue;
        }
        let Some(value) = after.strip_prefix('=') else {
            return Some(String::new());
        };
        let value = value.trim_start();
        return Some(match value.chars().next() {
            Some(q @ ('"' | '\'')) => value[1..].split(q).next().unwrap_or("").to_owned(),
            _ => value
                .split(|c: char| c.is_ascii_whitespace() || c == '>')
                .next()
                .unwrap_or("")
                .to_owned(),
        });
    }
    None
}

/// `Some(done)` when the `<li>` tag says it is a checklist item.
fn list_item_state(tag: &str) -> Option<bool> {
    for name in ["data-checked", "aria-checked"] {
        if let Some(value) = attr_value(tag, name) {
            return Some(value != "false" && value != "0");
        }
    }
    let class = attr_value(tag, "class")?;
    let words: Vec<&str> = class.split_whitespace().collect();
    let has = |names: &[&str]| words.iter().any(|w| names.contains(w));
    if has(&["unchecked", "todo", "task-open", "checklist-open"]) {
        Some(false)
    } else if has(&[
        "checked",
        "done",
        "completed",
        "task-done",
        "checklist-done",
    ]) {
        Some(true)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn checkbox_inputs_become_tasks() {
        let html = "<h1>Shopping</h1><ul>\
            <li><input type=\"checkbox\"> milk</li>\
            <li><input type=\"checkbox\" checked> eggs</li></ul><p>Thanks</p>";
        let md = html_to_markdown(html).unwrap();
        assert!(md.contains("- [ ] milk"), "{md}");
        assert!(md.contains("- [x] eggs"), "{md}");
        assert!(md.starts_with("# Shopping"));
        assert!(md.ends_with("Thanks"));
    }

    #[test]
    fn list_item_markers_become_tasks() {
        let html = "<ul><li class=\"checked\">a</li><li class=\"unchecked\">b</li>\
            <li data-checked=\"true\">c</li><li aria-checked=\"false\">d</li><li>e</li></ul>";
        let md = html_to_markdown(html).unwrap();
        assert_eq!(md, "- [x] a\n- [ ] b\n- [x] c\n- [ ] d\n- e");
    }

    #[test]
    fn a_checkbox_inside_a_marked_item_counts_once() {
        let html = "<ul><li class=\"done\"><input type=checkbox checked>x</li></ul>";
        assert_eq!(html_to_markdown(html).unwrap(), "- [x] x");
    }

    #[test]
    fn glyphs_become_tasks() {
        assert_eq!(
            tasks_from_glyphs("☐ one\n- ☑ two\n  ✅ three\nplain ☐ stays"),
            "- [ ] one\n- [x] two\n  - [x] three\nplain ☐ stays"
        );
    }

    #[test]
    fn glyphs_in_code_are_left_alone() {
        assert_eq!(tasks_from_glyphs("```\n☐ x\n```"), "```\n☐ x\n```");
    }

    #[test]
    fn the_subject_is_not_repeated() {
        let md = share_markdown(
            Some("Trip plan"),
            Some("<h1>Trip plan</h1><p>Book flights</p>"),
            None,
        )
        .unwrap();
        assert_eq!(md, "Book flights");
        let kept = share_markdown(Some("Other"), None, Some("Trip plan\nBook flights")).unwrap();
        assert_eq!(kept, "Trip plan\nBook flights");
    }

    #[test]
    fn text_is_used_when_there_is_no_html() {
        assert_eq!(
            share_markdown(None, None, Some("☐ call Sam\n☑ pay rent")).unwrap(),
            "- [ ] call Sam\n- [x] pay rent"
        );
        assert_eq!(share_markdown(None, Some("<p> </p>"), Some(" ")), None);
    }

    #[test]
    fn deep_html_is_refused() {
        let html = "<div>".repeat(MAX_DEPTH + 5);
        assert_eq!(html_to_markdown(&html), None);
    }

    #[test]
    fn picture_only_notes_say_drawing() {
        assert_eq!(say_picture_only("![](a.png)"), "(drawing)\n\n![](a.png)");
        assert_eq!(say_picture_only("hi\n\n![](a.png)"), "hi\n\n![](a.png)");
        assert_eq!(say_picture_only("plain"), "plain");
        assert_eq!(say_picture_only(""), "");
    }

    #[test]
    fn placeholders() {
        assert_eq!(placeholder(true), "(scan)");
        assert_eq!(placeholder(false), "(drawing)");
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 128, rng_seed: proptest::test_runner::RngSeed::Fixed(0x4e6f_7465), ..ProptestConfig::default() })]

        /// Any text converts without panicking, and no stand-in mark leaks.
        #[test]
        fn prop_no_mark_leaks(html in "[ -~\u{e000}\u{e001}\n]{0,200}") {
            if let Some(md) = html_to_markdown(&html) {
                prop_assert!(!md.contains(OPEN) && !md.contains(DONE));
            }
        }

        /// Converting what was converted again changes nothing about its
        /// tasks: glyph conversion is idempotent.
        #[test]
        fn prop_glyph_tasks_are_idempotent(text in "[ a-z☐☑✅\n*-]{0,80}") {
            let once = tasks_from_glyphs(&text);
            prop_assert_eq!(tasks_from_glyphs(&once), once);
        }

        /// Dropping the title never leaves more than the body had.
        #[test]
        fn prop_without_title_only_shortens(body in "[a-z \n]{0,60}", title in "[a-z ]{0,10}") {
            prop_assert!(without_title(&body, Some(&title)).len() <= body.len());
        }

        /// The task lines of one checklist keep their order and state.
        #[test]
        fn prop_checklist_state_round_trips(states in proptest::collection::vec(any::<bool>(), 1..12)) {
            let html: String = std::iter::once("<ul>".to_owned())
                .chain(states.iter().enumerate().map(|(n, s)| {
                    format!("<li><input type=\"checkbox\"{}> item{n}</li>", if *s { " checked" } else { "" })
                }))
                .chain(std::iter::once("</ul>".to_owned()))
                .collect();
            let md = html_to_markdown(&html).unwrap();
            let want: Vec<String> = states.iter().enumerate()
                .map(|(n, s)| format!("- [{}] item{n}", if *s { 'x' } else { ' ' }))
                .collect();
            prop_assert_eq!(md, want.join("\n"));
        }
    }
}
