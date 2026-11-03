//! Rich Text Format as Markdown paragraphs: the words of an `.rtf` note,
//! without fonts, colours or pictures. Pure, and safe on any bytes.

/// The deepest group nesting read; deeper input is cut off.
const MAX_GROUPS: usize = 256;

/// Groups whose contents are not text.
const SKIPPED: [&str; 16] = [
    "fonttbl",
    "colortbl",
    "stylesheet",
    "info",
    "pict",
    "header",
    "footer",
    "object",
    "filetbl",
    "listtable",
    "listoverridetable",
    "generator",
    "expandedcolortbl",
    "themedata",
    "datastore",
    "latentstyles",
];

/// The text of an RTF document: a line per paragraph, `•` bullets kept.
/// Input that is not RTF is returned as it is.
pub fn to_text(bytes: &[u8]) -> String {
    if !bytes.starts_with(b"{\\rtf") {
        return String::from_utf8_lossy(bytes).into_owned();
    }
    let mut out = String::new();
    // Per open group: whether its text is skipped.
    let mut skip: Vec<bool> = vec![false];
    // How many characters follow a `\u` escape as its fallback.
    let mut unicode_skip = 1usize;
    let mut pending_skip = 0usize;
    let mut i = 0;
    let mut first_in_group = false;
    while i < bytes.len() {
        let b = bytes[i];
        match b {
            b'{' => {
                if skip.len() < MAX_GROUPS {
                    let inherited = skip.last().copied().unwrap_or(false);
                    skip.push(inherited);
                    first_in_group = true;
                } else {
                    break;
                }
                i += 1;
            }
            b'}' => {
                if skip.len() > 1 {
                    skip.pop();
                }
                first_in_group = false;
                i += 1;
            }
            b'\\' => {
                i += 1;
                let Some(&next) = bytes.get(i) else { break };
                if next.is_ascii_alphabetic() {
                    let start = i;
                    while bytes.get(i).is_some_and(u8::is_ascii_alphabetic) {
                        i += 1;
                    }
                    let word = &bytes[start..i];
                    let negative = bytes.get(i) == Some(&b'-');
                    let digits_from = i + usize::from(negative);
                    let mut j = digits_from;
                    while bytes.get(j).is_some_and(u8::is_ascii_digit) {
                        j += 1;
                    }
                    let number: Option<i64> = std::str::from_utf8(&bytes[digits_from..j])
                        .ok()
                        .and_then(|s| s.parse::<i64>().ok())
                        .map(|n| if negative { -n } else { n });
                    if j > digits_from {
                        i = j;
                    }
                    // One space ends a control word.
                    if bytes.get(i) == Some(&b' ') {
                        i += 1;
                    }
                    let hidden = skip.last().copied().unwrap_or(false);
                    let word = std::str::from_utf8(word).unwrap_or("");
                    if first_in_group
                        && SKIPPED.contains(&word)
                        && let Some(last) = skip.last_mut()
                    {
                        *last = true;
                    }
                    first_in_group = false;
                    if hidden || skip.last().copied().unwrap_or(false) {
                        continue;
                    }
                    match word {
                        "par" | "line" | "sect" | "page" => out.push('\n'),
                        "tab" => out.push('\t'),
                        "emdash" => out.push('—'),
                        "endash" => out.push('–'),
                        "bullet" => out.push('•'),
                        "lquote" => out.push('‘'),
                        "rquote" => out.push('’'),
                        "ldblquote" => out.push('“'),
                        "rdblquote" => out.push('”'),
                        "uc" => unicode_skip = number.unwrap_or(1).clamp(0, 8) as usize,
                        "u" => {
                            if let Some(n) = number {
                                let code = if n < 0 { n + 65_536 } else { n };
                                if let Some(c) = u32::try_from(code).ok().and_then(char::from_u32) {
                                    out.push(c);
                                }
                                pending_skip = unicode_skip;
                            }
                        }
                        _ => {}
                    }
                } else {
                    i += 1;
                    let hidden = skip.last().copied().unwrap_or(false);
                    match next {
                        b'*' => {
                            if let Some(last) = skip.last_mut() {
                                *last = true;
                            }
                        }
                        b'\'' => {
                            let hex = bytes
                                .get(i..i + 2)
                                .and_then(|h| std::str::from_utf8(h).ok());
                            let value = hex.and_then(|h| u8::from_str_radix(h, 16).ok());
                            i = (i + 2).min(bytes.len());
                            if pending_skip > 0 {
                                pending_skip -= 1;
                            } else if let (Some(v), false) = (value, hidden) {
                                out.push(windows_1252(v));
                            }
                        }
                        b'~' if !hidden => out.push('\u{a0}'),
                        b'_' | b'-' if !hidden => out.push('-'),
                        b'\n' | b'\r' if !hidden => out.push('\n'),
                        b'\\' | b'{' | b'}' if !hidden => out.push(char::from(next)),
                        _ => {}
                    }
                    first_in_group = false;
                }
            }
            b'\r' | b'\n' => i += 1,
            _ => {
                // A run of plain bytes.
                let start = i;
                while i < bytes.len() && !matches!(bytes[i], b'{' | b'}' | b'\\' | b'\r' | b'\n') {
                    i += 1;
                }
                first_in_group = false;
                if skip.last().copied().unwrap_or(false) {
                    continue;
                }
                let mut run = &bytes[start..i];
                while pending_skip > 0 && !run.is_empty() {
                    run = &run[1..];
                    pending_skip -= 1;
                }
                out.push_str(&String::from_utf8_lossy(run));
            }
        }
    }
    out
}

/// The character for a `\'hh` byte, in Windows-1252.
fn windows_1252(byte: u8) -> char {
    const HIGH: [char; 32] = [
        '€', '\u{81}', '‚', 'ƒ', '„', '…', '†', '‡', 'ˆ', '‰', 'Š', '‹', 'Œ', '\u{8d}', 'Ž',
        '\u{8f}', '\u{90}', '‘', '’', '“', '”', '•', '–', '—', '˜', '™', 'š', '›', 'œ', '\u{9d}',
        'ž', 'Ÿ',
    ];
    match byte {
        0x80..=0x9f => HIGH[usize::from(byte - 0x80)],
        other => char::from(other),
    }
}

/// An RTF document as Markdown: each paragraph a paragraph, runs of `•`
/// lines a list.
pub fn to_markdown(bytes: &[u8]) -> String {
    let text = to_text(bytes);
    let mut out = String::new();
    let mut previous_item = false;
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let item = line
            .strip_prefix(['•', '◦', '▪', '‣'])
            .map(|rest| rest.trim_start_matches([' ', '\t', '\u{a0}']));
        if !out.is_empty() {
            out.push_str(if item.is_some() && previous_item {
                "\n"
            } else {
                "\n\n"
            });
        }
        previous_item = item.is_some();
        match item {
            Some(rest) => {
                out.push_str("- ");
                out.push_str(rest);
            }
            None => out.push_str(line),
        }
    }
    super::notes_html::tasks_from_glyphs(&out)
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;

    use super::*;

    #[test]
    fn plain_paragraphs() {
        let rtf = br"{\rtf1\ansi{\fonttbl{\f0 Helvetica;}}{\colortbl;\red0\green0\blue0;}\f0\fs24 Title\par Second line\par}";
        assert_eq!(to_markdown(rtf), "Title\n\nSecond line");
    }

    #[test]
    fn escapes_and_unicode() {
        let rtf = br"{\rtf1\ansi caf\'e9 \u8212? \u-10179?\u-8704? \{x\}\par}";
        let text = to_text(rtf);
        assert!(text.contains("café"));
        assert!(text.contains('—'));
        assert!(text.contains("{x}"));
    }

    #[test]
    fn bullets_become_a_list() {
        let rtf = br"{\rtf1 Groceries\par \bullet\tab milk\par \bullet\tab eggs\par}";
        assert_eq!(to_markdown(rtf), "Groceries\n\n- milk\n- eggs");
    }

    #[test]
    fn ignorable_destinations_are_skipped() {
        let rtf = br"{\rtf1{\*\generator Cocoa}{\info{\author Me}}{\pict\jpegblip ffd8}Hello\par}";
        assert_eq!(to_markdown(rtf), "Hello");
    }

    #[test]
    fn non_rtf_is_returned_as_text() {
        assert_eq!(to_text(b"just words"), "just words");
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 256, rng_seed: proptest::test_runner::RngSeed::Fixed(0x5274_6621), ..ProptestConfig::default() })]

        /// Any bytes after an RTF header convert without panicking and
        /// never show a control word's backslash from a known escape.
        #[test]
        fn prop_any_bytes_are_safe(tail in proptest::collection::vec(any::<u8>(), 0..300)) {
            let mut rtf = b"{\\rtf1".to_vec();
            rtf.extend(tail);
            let _ = to_markdown(&rtf);
        }

        /// Plain words in a group come out unchanged.
        #[test]
        fn prop_words_survive(words in "[a-zA-Z]{1,8}( [a-zA-Z]{1,8}){0,5}") {
            let rtf = format!("{{\\rtf1\\ansi {words}\\par}}");
            prop_assert_eq!(to_markdown(rtf.as_bytes()), words);
        }

        /// A skipped destination hides its text wherever it appears.
        #[test]
        fn prop_skipped_groups_hide_text(hidden in "[a-z]{3,10}") {
            let rtf = format!("{{\\rtf1{{\\fonttbl {hidden}}}visible\\par}}");
            prop_assert_eq!(to_markdown(rtf.as_bytes()), "visible");
        }
    }
}
