//! What a paste puts in the note: a web address pasted over selected words
//! links them, formatted text from a browser or a document comes in as
//! Markdown, and a block (images, say) goes in as a paragraph of its own.
//! Pure apart from [`html_to_markdown`], which parses HTML with `htmd`.

use std::ops::Range;

use super::Edit;

/// Whether `text` is a single web address. The share sheet's inbox rules
/// decide the same question, so there is one answer (`kanso_engine::capture`).
pub use crate::capture::is_url;

/// A web address pasted over selected words on one line makes them a
/// link: `[words](address)`, with the cursor after it. `None` when the
/// paste isn't one address, nothing is selected, the selection spans lines
/// or is itself an address.
pub fn link_over_selection(text: &str, selection: Range<usize>, pasted: &str) -> Option<Edit> {
    if selection.is_empty() || selection.end > text.len() || !is_url(pasted) {
        return None;
    }
    let selected = text.get(selection.clone())?;
    if selected.contains('\n') || is_url(selected) || selected.trim().is_empty() {
        return None;
    }
    let link = format!("[{selected}]({})", pasted.trim());
    let end = selection.start + link.len();
    Some(Edit {
        range: selection,
        text: link,
        selection: end..end,
    })
}

/// Tags that make HTML worth turning into Markdown. Code editors put their
/// text on the clipboard as styled `<div>`s and `<span>`s, which say
/// nothing the plain text doesn't, and would come in escaped.
const STRUCTURE: [&str; 17] = [
    "<a ",
    "<b>",
    "<b ",
    "<strong",
    "<em",
    "<i>",
    "<h1",
    "<h2",
    "<h3",
    "<h4",
    "<ul",
    "<ol",
    "<table",
    "<blockquote",
    "<pre",
    "<img",
    "<code",
];

/// Whether the HTML on the clipboard has structure the plain text lacks.
pub fn worth_converting(html: &str) -> bool {
    let lower = html.to_ascii_lowercase();
    STRUCTURE.iter().any(|tag| lower.contains(tag))
}

/// HTML as Markdown in Kanso's house style: ATX headings, `-` bullets,
/// fenced code, `---` rules. Runs of blank lines collapse to one, and the
/// ends are trimmed. `None` when nothing is left.
pub fn html_to_markdown(html: &str) -> Option<String> {
    use htmd::options::{BrStyle, BulletListMarker, HrStyle, Options};
    let converter = htmd::HtmlToMarkdown::builder()
        .options(Options {
            bullet_list_marker: BulletListMarker::Dash,
            hr_style: HrStyle::Dashes,
            br_style: BrStyle::Backslash,
            ul_bullet_spacing: 1,
            ol_number_spacing: 1,
            ..Options::default()
        })
        .skip_tags(vec!["script", "style", "head", "meta", "title"])
        .build();
    let markdown = converter.convert(html).ok()?;
    let tidy = tidy(&markdown);
    (!tidy.is_empty()).then_some(tidy)
}

/// Trailing spaces off each line, at most one blank line in a row, and no
/// blank lines at either end.
pub fn tidy(markdown: &str) -> String {
    let mut out = String::with_capacity(markdown.len());
    let mut blank = 0;
    for line in markdown.trim_matches('\n').lines() {
        let line = line.trim_end();
        if line.is_empty() {
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

/// Puts `block` (images, say) over `range` as a paragraph of its own: a
/// blank line from the text before and after it, so the preview shows it
/// alone and offers its sizes. The cursor goes after it.
pub fn block_insert(text: &str, range: Range<usize>, block: &str) -> Edit {
    let start = range.start.min(text.len());
    let end = range.end.clamp(start, text.len());
    let before = &text[..start];
    let after = &text[end..];
    let lead = if before.is_empty() || before.ends_with("\n\n") {
        ""
    } else if before.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    let trail = if after.is_empty() || after.starts_with("\n\n") {
        ""
    } else if after.starts_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    let cursor = start + lead.len() + block.len();
    Edit {
        range: start..end,
        text: format!("{lead}{block}{trail}"),
        selection: cursor..cursor,
    }
}

/// An `<img>` tag on its own: `<img src="<attachment link>" width="320">`.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImgTag {
    /// The tag's `src`, as written.
    pub src: String,
    /// The tag's `alt`, when it has one.
    pub alt: String,
    /// The tag's `width`, when it sets one.
    pub width: Option<u32>,
}

/// The address, words and width of an `<img>` tag, when `html` is one.
pub fn img_tag(html: &str) -> Option<ImgTag> {
    let tag = html.trim();
    let inner = tag.strip_prefix("<img")?.strip_suffix('>')?;
    let inner = inner.strip_suffix('/').unwrap_or(inner);
    if inner.contains('<') || inner.contains('>') {
        return None;
    }
    let attr = |name: &str| -> Option<String> {
        let at = inner.find(&format!("{name}="))?;
        let before = inner[..at].chars().next_back();
        if before.is_some_and(|c| !c.is_whitespace()) {
            return None;
        }
        let rest = &inner[at + name.len() + 1..];
        let (quote, rest) = match rest.chars().next()? {
            q @ ('"' | '\'') => (Some(q), &rest[1..]),
            _ => (None, rest),
        };
        let end = match quote {
            Some(q) => rest.find(q)?,
            None => rest.find(char::is_whitespace).unwrap_or(rest.len()),
        };
        Some(rest[..end].to_owned())
    };
    let src = attr("src")?;
    let width = attr("width").and_then(|w| w.trim_end_matches("px").parse().ok());
    let alt = attr("alt")
        .unwrap_or_default()
        .replace("&quot;", "\"")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&amp;", "&");
    Some(ImgTag { src, alt, width })
}

/// The Markdown for attachment `src` shown `width` points wide, or at its
/// own size: what the preview's size buttons write.
pub fn sized_image(src: &str, alt: &str, width: Option<u32>) -> String {
    match width {
        Some(width) => {
            let alt = alt
                .replace('&', "&amp;")
                .replace('"', "&quot;")
                .replace('<', "&lt;")
                .replace('>', "&gt;");
            if alt.is_empty() {
                format!("<img src=\"{src}\" width=\"{width}\">")
            } else {
                format!("<img src=\"{src}\" alt=\"{alt}\" width=\"{width}\">")
            }
        }
        None => format!("![{alt}]({src})"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    #[test]
    fn blocks_stand_alone() {
        let image = "![](inkdrop://file:a)";
        let at = |text: &str, i: usize| block_insert(text, i..i, image).apply(text);
        assert_eq!(at("", 0), image);
        assert_eq!(at("words", 5), format!("words\n\n{image}"));
        assert_eq!(at("words\n", 6), format!("words\n\n{image}"));
        assert_eq!(at("a b", 1), format!("a\n\n{image}\n\n b"));
        assert_eq!(at("a\n\nb", 2), format!("a\n\n{image}\n\nb"));
        let edit = block_insert("ab", 1..1, image);
        assert_eq!(edit.selection.start, 3 + image.len());
    }

    proptest! {
        #![proptest_config(config())]
        /// The block ends up a paragraph of its own, only newlines are
        /// added, and the cursor is after it.
        #[test]
        fn prop_blocks_are_paragraphs(text in "[a-z\n]{0,12}", i in any::<prop::sample::Index>()) {
            let image = "![](inkdrop://file:a)";
            let at = i.index(text.len() + 1);
            let edit = block_insert(&text, at..at, image);
            let out = edit.apply(&text);
            let start = out.find(image).unwrap();
            let end = start + image.len();
            prop_assert!(start == 0 || out[..start].ends_with("\n\n"));
            prop_assert!(end == out.len() || out[end..].starts_with("\n\n"));
            prop_assert_eq!(edit.selection, end..end);
            prop_assert_eq!(out.replace(image, "").replace('\n', ""), text.replace('\n', ""));
        }
    }

    #[test]
    fn urls() {
        assert!(is_url("https://kanso.app"));
        assert!(is_url("  http://a.b/c?d=e  "));
        assert!(is_url("inkdrop://note/abc"));
        assert!(!is_url("https://"));
        assert!(!is_url("see https://kanso.app"));
        assert!(!is_url("kanso.app"));
    }

    #[test]
    fn a_url_over_words_links_them() {
        let edit = link_over_selection("read the docs", 9..13, "https://d.io").unwrap();
        assert_eq!(edit.apply("read the docs"), "read the [docs](https://d.io)");
        assert!(link_over_selection("read", 0..0, "https://d.io").is_none());
        assert!(link_over_selection("a\nb", 0..3, "https://d.io").is_none());
        assert!(link_over_selection("read", 0..4, "not a url").is_none());
        assert!(link_over_selection("https://x.io", 0..12, "https://d.io").is_none());
    }

    #[test]
    fn html_becomes_markdown() {
        let md = html_to_markdown(
            "<meta charset='utf-8'><h2>Title</h2><p>Some <b>bold</b> and <a href=\"https://k.app\">a link</a>.</p><ul><li>one</li><li>two</li></ul>",
        )
        .unwrap();
        assert_eq!(
            md,
            "## Title\n\nSome **bold** and [a link](https://k.app).\n\n- one\n- two"
        );
        let table = html_to_markdown(
            "<table><thead><tr><th>A</th><th>B</th></tr></thead><tbody><tr><td>1</td><td>2</td></tr></tbody></table>",
        )
        .unwrap();
        assert!(table.contains("| A") && table.contains("| 1"), "{table}");
        assert_eq!(html_to_markdown("<div></div>"), None);
    }

    #[test]
    fn code_editors_html_is_left_alone() {
        assert!(!worth_converting(
            "<meta charset='utf-8'><div style=\"color: #d4d4d4;\"><div><span>fn</span> main() {}</div></div>"
        ));
        assert!(worth_converting("<p>a <STRONG>b</STRONG></p>"));
    }

    #[test]
    fn img_tags() {
        let tag = |src: &str, alt: &str, width| {
            Some(ImgTag {
                src: src.into(),
                alt: alt.into(),
                width,
            })
        };
        assert_eq!(
            img_tag("<img src=\"inkdrop://file:x\" width=\"320\">"),
            tag("inkdrop://file:x", "", Some(320))
        );
        assert_eq!(
            img_tag("<img width='200px' src='a.png' alt='A cat' />"),
            tag("a.png", "A cat", Some(200))
        );
        assert_eq!(img_tag("<img src=a.png>"), tag("a.png", "", None));
        assert_eq!(img_tag("<p>x</p>"), None);
        assert_eq!(img_tag("<img data-src=\"a\">"), None);
    }

    proptest! {
        #![proptest_config(config())]

        /// Round trip: the size buttons' tag reads back as the same address
        /// and width.
        #[test]
        fn prop_sized_images_read_back(id in "[a-zA-Z0-9_-]{1,12}", width in 1u32..4000, alt in "[a-z \"<>&]{0,8}") {
            let src = format!("inkdrop://file:{id}");
            let tag = sized_image(&src, &alt, Some(width));
            prop_assert_eq!(img_tag(&tag), Some(ImgTag { src: src.clone(), alt: alt.clone(), width: Some(width) }));
            prop_assert_eq!(sized_image(&src, "", None), format!("![]({src})"));
        }

        /// The link keeps the selected words and the address, and the rest
        /// of the text.
        #[test]
        fn prop_links_keep_the_words(before in "[a-z ]{0,8}", words in "[a-z][a-z ]{0,8}", after in "[a-z ]{0,8}", path in "[a-z/]{1,10}") {
            let text = format!("{before}{words}{after}");
            let url = format!("https://example.com/{path}");
            let sel = before.len()..before.len() + words.len();
            let edit = link_over_selection(&text, sel, &url).unwrap();
            prop_assert_eq!(edit.apply(&text), format!("{before}[{words}]({url}){after}"));
        }

        /// Tidy text has no blank line at either end, none doubled, and no
        /// trailing spaces; tidying again changes nothing.
        #[test]
        fn prop_tidy_is_tidy(text in "([a-z ]{0,6}\n{0,3}){0,8}") {
            let out = tidy(&text);
            prop_assert!(!out.starts_with('\n') && !out.ends_with('\n'));
            prop_assert!(!out.contains("\n\n\n"));
            prop_assert!(out.lines().all(|l| l == l.trim_end()));
            prop_assert_eq!(tidy(&out), out.clone());
        }
    }
}
