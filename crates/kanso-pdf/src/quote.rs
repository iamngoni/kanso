//! The quote a highlight writes into the note, and where it goes.

use std::ops::Range;

use crate::link::PdfLink;

/// One text replacement in a body.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Edit {
    pub range: Range<usize>,
    pub insert: String,
}

impl Edit {
    pub fn apply(&self, body: &str) -> String {
        let mut out = String::with_capacity(body.len() + self.insert.len());
        out.push_str(&body[..self.range.start]);
        out.push_str(&self.insert);
        out.push_str(&body[self.range.end..]);
        out
    }
}

/// The Markdown quote: the words, then a page link.
///
/// ```text
/// > Revenue rose 12 percent.
/// >
/// > — [p. 4](inkdrop://file:abc123#page=4&h=h7k2m9)
/// ```
///
/// The words are the PDF's, so they are written as text: Markdown and HTML in
/// them (`[x](https://…)`, `<img src=…>`, a leading `#` or `-`, `$x$` math,
/// a bare `https://…` or `a@b.c` that would become a link) is escaped and
/// reads as the characters it is, never as a link, an image the preview
/// would fetch, a list, a heading or code. The words are one line, so no
/// fence can open in them.
pub fn render_quote(text: &str, link: &PdfLink) -> String {
    let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
    let page = link.page.unwrap_or(1);
    format!(
        "> {}\n>\n> — [p. {page}]({})",
        escape_markdown(&words),
        link.to_url()
    )
}

/// `text` (one line) with every character that could start Markdown or
/// HTML escaped: the inline markers anywhere, the block markers at its
/// start, and what would make a bare address a link.
pub fn escape_markdown(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len() + 8);
    for (i, &c) in chars.iter().enumerate() {
        let next = |k: usize| chars.get(i + k).copied();
        // A list item, a rule or a setext line at the start: `- `, `+ `,
        // `1. `, `2) `, `===`.
        let numbered = (1..=9).contains(&i)
            && chars[..i].iter().all(char::is_ascii_digit)
            && matches!(c, '.' | ')')
            && next(1).is_none_or(|n| n == ' ');
        let leading = i == 0 && matches!(c, '-' | '+' | '=');
        // `https://…` and `www.…` become links in GFM, `a@b.c` a mail link.
        let scheme = c == ':' && next(1) == Some('/') && next(2) == Some('/');
        let www = c == '.'
            && i >= 3
            && chars[i - 3..i].iter().all(|w| w.eq_ignore_ascii_case(&'w'))
            && (i == 3 || !chars[i - 4].is_alphanumeric());
        if leading
            || numbered
            || scheme
            || www
            || matches!(
                c,
                '\\' | '`'
                    | '*'
                    | '_'
                    | '['
                    | ']'
                    | '<'
                    | '>'
                    | '#'
                    | '!'
                    | '|'
                    | '~'
                    | '&'
                    | '$'
                    | '@'
            )
        {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// A heading's text without its optional closing `#`s, which only close it
/// after a space (`# C\#` keeps its escaped `#`).
fn heading_text(rest: &str) -> &str {
    let t = rest.trim();
    let open = t.trim_end_matches('#');
    if open.is_empty() {
        return "";
    }
    if open.len() < t.len() && open.ends_with([' ', '\t']) {
        return open.trim_end();
    }
    t
}

#[derive(Clone, Copy)]
struct Heading {
    start: usize,
    end: usize,
    level: usize,
}

fn headings(body: &str) -> Vec<(Heading, String)> {
    let mut out = Vec::new();
    let mut fence: Option<(char, usize)> = None;
    let mut pos = 0;
    for raw in body.split_inclusive('\n') {
        let line = raw.trim_end_matches(['\n', '\r']);
        let t = line.trim_start_matches(' ');
        let indent = line.len() - t.len();
        let c = t.chars().next();
        let run = c.map_or(0, |c| t.chars().take_while(|x| *x == c).count());
        match fence {
            Some((fc, n)) => {
                if indent <= 3 && c == Some(fc) && run >= n && t[run..].trim().is_empty() {
                    fence = None;
                }
            }
            None => {
                if indent <= 3 && matches!(c, Some('`' | '~')) && run >= 3 {
                    fence = Some((c.unwrap_or('`'), run));
                } else if indent <= 3 {
                    let level = t.chars().take_while(|x| *x == '#').count();
                    if (1..=6).contains(&level) && t[level..].starts_with([' ', '\t']) {
                        let title = heading_text(&t[level..]).to_string();
                        out.push((
                            Heading {
                                start: pos,
                                end: pos + line.len(),
                                level,
                            },
                            title,
                        ));
                    }
                }
            }
        }
        pos += raw.len();
    }
    out
}

/// Where the section that starts at heading `i` ends: before the next heading
/// at the same or a higher level, else the end of the body.
fn section_end(body: &str, hs: &[(Heading, String)], i: usize) -> usize {
    hs[i + 1..]
        .iter()
        .find(|(h, _)| h.level <= hs[i].0.level)
        .map_or(body.len(), |(h, _)| h.start)
}

/// The edit that puts `quote` under `## Highlights` › `### <pdf_name>`,
/// creating either heading that is missing at the end of the note. Pure: it
/// returns the edit and touches nothing.
pub fn append_quote(body: &str, pdf_name: &str, quote: &str) -> Edit {
    // The name is the attachment's, as text: `<img …>` or `[x](…)` in it is
    // not markup in the heading.
    let name = escape_markdown(&pdf_name.split_whitespace().collect::<Vec<_>>().join(" "));
    let hs = headings(body);
    let top = hs
        .iter()
        .position(|(h, t)| h.level == 2 && t.eq_ignore_ascii_case("highlights"));
    let at_end = |from: usize, end: usize, text: String| {
        // Put it right after the section's last content, ahead of any blank
        // lines that separate it from what follows.
        let at = from + body[from..end].trim_end().len();
        Edit {
            range: at..at,
            insert: text,
        }
    };
    if let Some(t) = top {
        let top_end = section_end(body, &hs, t);
        let sub = hs[t + 1..]
            .iter()
            .position(|(h, title)| {
                h.start < top_end && h.level == 3 && title.eq_ignore_ascii_case(&name)
            })
            .map(|k| t + 1 + k);
        if let Some(s) = sub {
            let end = section_end(body, &hs, s);
            return at_end(hs[s].0.end, end, format!("\n\n{quote}"));
        }
        return at_end(hs[t].0.end, top_end, format!("\n\n### {name}\n\n{quote}"));
    }
    let end = body.trim_end().len();
    let text = if end == 0 {
        format!("## Highlights\n\n### {name}\n\n{quote}\n")
    } else {
        format!("\n\n## Highlights\n\n### {name}\n\n{quote}")
    };
    Edit {
        range: end..end,
        insert: text,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn link() -> PdfLink {
        PdfLink::parse("inkdrop://file:abc123#page=4&h=h7k2m9").unwrap()
    }

    #[test]
    fn the_quote_matches_the_spec() {
        assert_eq!(
            render_quote("Revenue rose  12 percent\non the quarter.", &link()),
            "> Revenue rose 12 percent on the quarter.\n>\n> — [p. 4](inkdrop://file:abc123#page=4&h=h7k2m9)"
        );
    }

    #[test]
    fn pdf_text_is_written_as_text() {
        let q = render_quote(
            "# Hi [x](https://e.test) <img src=https://e.test/b> *a*",
            &link(),
        );
        let first = q.lines().next().unwrap();
        assert_eq!(
            first,
            r"> \# Hi \[x\](https\://e.test) \<img src=https\://e.test/b\> \*a\*"
        );
    }

    #[test]
    fn creates_both_headings_at_the_end() {
        let q = render_quote("Words", &link());
        let out = append_quote("# Title\n\nbody\n", "report.pdf", &q).apply("# Title\n\nbody\n");
        assert_eq!(
            out,
            format!("# Title\n\nbody\n\n## Highlights\n\n### report.pdf\n\n{q}\n")
        );
    }

    #[test]
    fn an_empty_note_gets_just_the_section() {
        let q = render_quote("W", &link());
        let out = append_quote("", "a.pdf", &q).apply("");
        assert!(out.starts_with("## Highlights\n\n### a.pdf\n\n> W"));
    }

    #[test]
    fn appends_to_the_matching_subsection() {
        let body =
            "## Highlights\n\n### a.pdf\n\n> one\n\n### b.pdf\n\n> two\n\n## Other\n\ntext\n";
        let out = append_quote(body, "a.pdf", "> three").apply(body);
        assert_eq!(
            out,
            "## Highlights\n\n### a.pdf\n\n> one\n\n> three\n\n### b.pdf\n\n> two\n\n## Other\n\ntext\n"
        );
    }

    #[test]
    fn adds_a_subsection_inside_an_existing_highlights_section() {
        let body = "## Highlights\n\n### a.pdf\n\n> one\n\n## Other\n\ntext\n";
        let out = append_quote(body, "b.pdf", "> two").apply(body);
        assert_eq!(
            out,
            "## Highlights\n\n### a.pdf\n\n> one\n\n### b.pdf\n\n> two\n\n## Other\n\ntext\n"
        );
    }

    #[test]
    fn a_heading_inside_code_is_not_a_heading() {
        let body = "```\n## Highlights\n```\n";
        let out = append_quote(body, "a.pdf", "> q").apply(body);
        assert!(
            out.ends_with("```\n\n## Highlights\n\n### a.pdf\n\n> q\n"),
            "{out}"
        );
    }

    #[test]
    fn an_empty_highlights_heading_gets_its_subsection() {
        let body = "## Highlights\n";
        let out = append_quote(body, "a.pdf", "> q").apply(body);
        assert_eq!(out, "## Highlights\n\n### a.pdf\n\n> q\n");
    }

    proptest! {
        #[test]
        fn the_quote_always_lands_once_and_the_rest_is_kept(
            body in "[#a-z`~\\n >\\-]{0,120}",
            name in "[a-z]{1,8}\\.pdf",
        ) {
            let q = "> QUOTE-MARK";
            let e = append_quote(&body, &name, q);
            prop_assert!(e.range.start == e.range.end && e.range.end <= body.len());
            prop_assert!(body.is_char_boundary(e.range.start));
            let out = e.apply(&body);
            prop_assert_eq!(out.matches(q).count(), 1);
            // Nothing already in the note is lost or reordered.
            prop_assert!(out.starts_with(&body[..e.range.start]));
            prop_assert!(out.ends_with(&body[e.range.end..]));
            prop_assert_eq!(out.len(), body.len() + e.insert.len());
        }

        /// Whatever the PDF's words are, the preview's parser (GFM with
        /// math, as Kanso reads notes) reads the quote's first line as
        /// exactly those words in plain text: no link, emphasis, code, math,
        /// HTML, list or heading, and the second as the page link alone.
        #[test]
        fn the_words_read_back_as_plain_text(
            text in prop_oneof![
                "[a-zA-Z0-9 .:/@#*_`~<>!()|&$=+^%{}\\[\\]\\\\-]{0,80}",
                "(1[.)] |- |\\+ |# |> |www\\.|https?://|a@b\\.c |```|~~~|\\$\\$|<!--|\\[\\[|\\|)[a-z .]{0,20}",
                "\\PC{0,120}",
            ],
        ) {
            use markdown::mdast::Node;
            let q = render_quote(&text, &link());
            let mut options = markdown::ParseOptions::gfm();
            options.constructs.math_text = true;
            options.constructs.math_flow = true;
            let tree = markdown::to_mdast(&q, &options).unwrap();
            let Node::Root(root) = tree else { unreachable!() };
            prop_assert_eq!(root.children.len(), 1, "{:?}", q);
            let Node::Blockquote(quote) = &root.children[0] else {
                return Err(TestCaseError::fail(format!("not a quote: {q:?}")));
            };
            let words = text.split_whitespace().collect::<Vec<_>>().join(" ");
            let paragraphs: Vec<&Vec<Node>> = quote
                .children
                .iter()
                .map(|n| match n {
                    Node::Paragraph(p) => Ok(&p.children),
                    other => Err(format!("{other:?} in {q:?}")),
                })
                .collect::<Result<_, _>>()
                .map_err(TestCaseError::fail)?;
            let (first, last) = if words.is_empty() {
                prop_assert_eq!(paragraphs.len(), 1, "{:?}", q);
                (None, paragraphs[0])
            } else {
                prop_assert_eq!(paragraphs.len(), 2, "{:?}", q);
                (Some(paragraphs[0]), paragraphs[1])
            };
            if let Some(first) = first {
                let mut read = String::new();
                for n in first {
                    match n {
                        Node::Text(t) => read.push_str(&t.value),
                        other => {
                            return Err(TestCaseError::fail(format!("{other:?} in {q:?}")));
                        }
                    }
                }
                prop_assert_eq!(read, words);
            }
            let links: Vec<&str> = last
                .iter()
                .filter_map(|n| match n {
                    Node::Link(l) => Some(l.url.as_str()),
                    _ => None,
                })
                .collect();
            prop_assert_eq!(links, vec!["inkdrop://file:abc123#page=4&h=h7k2m9"]);
        }

        /// A PDF's name is text in its heading, and a second quote finds the
        /// same subsection.
        #[test]
        fn the_pdf_name_is_text_and_its_section_is_found_again(
            name in "[a-zA-Z0-9 .#*_`<>!\\[\\]()&$-]{1,30}",
        ) {
            let one = append_quote("", &name, "> one").apply("");
            let two = append_quote(&one, &name, "> two").apply(&one);
            prop_assert_eq!(two.matches("### ").count(), 1, "{}", two);
            let mut options = markdown::ParseOptions::gfm();
            options.constructs.math_text = true;
            let tree = markdown::to_mdast(&two, &options).unwrap();
            let markdown::mdast::Node::Root(root) = tree else { unreachable!() };
            let heading = root.children.iter().find_map(|n| match n {
                markdown::mdast::Node::Heading(h) if h.depth == 3 => Some(h),
                _ => None,
            });
            let heading = heading.expect("the subsection heading");
            let all_text = heading
                .children
                .iter()
                .all(|n| matches!(n, markdown::mdast::Node::Text(_)));
            prop_assert!(all_text, "{:?}", heading.children);
        }

        #[test]
        fn quotes_are_one_paragraph_of_blockquote(text in "\\PC{0,200}") {
            let q = render_quote(&text, &link());
            prop_assert!(q.lines().all(|l| l.starts_with('>')));
            prop_assert_eq!(q.lines().count(), 3);
        }
    }
}
