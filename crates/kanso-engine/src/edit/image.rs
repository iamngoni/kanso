//! Image widths, as a note writes them: `![alt](src)` at the image's own
//! size, `![alt|300](src)` (Obsidian's way) or `<img src=… width="300">`.
//! The preview's size buttons and drag handle, and "Resize image…" in the
//! palette, rewrite an image's line through here. Also where "New drawing"
//! puts its block. Pure.

use std::ops::Range;

use kanso_render::html::split_image_width;

use super::Edit;
use super::paste;

/// The widths an attached image can be given with the size buttons, besides
/// its own: small and medium.
pub const IMAGE_SIZES: [(&str, Option<u32>); 3] =
    [("Small", Some(240)), ("Medium", Some(480)), ("Full", None)];

/// The narrowest an image can be dragged or typed to.
pub const MIN_WIDTH: u32 = 48;
/// The widest a typed width can be.
pub const MAX_WIDTH: u32 = 4000;

/// An image on a line of its own.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct LineImage {
    /// The image's address.
    pub src: String,
    /// Without any `|300`.
    pub alt: String,
    /// The width written after `|`, when the image sets one.
    pub width: Option<u32>,
    /// Written `![alt|300](src)`, so a new width is written the same way.
    pub bar: bool,
}

/// The image `line` is, when it is only one (spaces around it aside):
/// `![alt](src)`, `![alt|300](src)` or an `<img>` tag.
pub fn line_image(line: &str) -> Option<LineImage> {
    let line = line.trim();
    if line.starts_with("<img") {
        let tag = paste::img_tag(line)?;
        return Some(LineImage {
            src: tag.src,
            alt: tag.alt,
            width: tag.width,
            bar: false,
        });
    }
    let rest = line.strip_prefix("![")?.strip_suffix(')')?;
    let (alt, src) = rest.rsplit_once("](")?;
    let src = src.trim();
    if src.is_empty() || src.contains(char::is_whitespace) || alt.contains(['[', ']']) {
        return None;
    }
    let (text, width) = split_image_width(alt);
    Some(LineImage {
        src: src.to_owned(),
        alt: text.to_owned(),
        width,
        bar: width.is_some(),
    })
}

/// `image` written at `width` (`None`: its own size), in the form it was
/// written in: `![alt|300](src)` stays a bar, anything else becomes an
/// `<img>` tag as the preview's size buttons write it.
pub fn written(image: &LineImage, width: Option<u32>) -> String {
    match (image.bar, width) {
        (true, Some(width)) => format!("![{}|{width}]({})", image.alt, image.src),
        _ => paste::sized_image(&image.src, &image.alt, width),
    }
}

/// The line holding byte `at` in `text`, without its newline.
pub fn line_at(text: &str, at: usize) -> Range<usize> {
    let at = at.min(text.len());
    let start = text[..at].rfind('\n').map_or(0, |i| i + 1);
    let end = text[at..].find('\n').map_or(text.len(), |i| at + i);
    start..end
}

/// The image on the line holding `at`, and that line's range.
pub fn image_at(text: &str, at: usize) -> Option<(Range<usize>, LineImage)> {
    let range = line_at(text, at);
    let image = line_image(&text[range.clone()])?;
    Some((range, image))
}

/// The edit giving the image on `at`'s line `width`: one undo step, with
/// the cursor after the image.
pub fn resize_at(text: &str, at: usize, width: Option<u32>) -> Option<Edit> {
    let (range, image) = image_at(text, at)?;
    let line = &text[range.clone()];
    let indent = &line[..line.len() - line.trim_start().len()];
    let new = format!("{indent}{}", written(&image, width));
    let end = range.start + new.len();
    Some(Edit {
        range,
        text: new,
        selection: end..end,
    })
}

/// How wide an image dragged by its right edge to `pointer_x` shows, when
/// its left edge is at `left` and it may be up to `max` wide: whole pixels,
/// never under [`MIN_WIDTH`] or over `max`.
pub fn drag_width(pointer_x: f32, left: f32, max: f32) -> u32 {
    let max = max.max(MIN_WIDTH as f32);
    let width = (pointer_x - left).clamp(MIN_WIDTH as f32, max);
    width.round() as u32
}

/// A width typed in "Resize image…": digits, maybe with "px". Pure.
pub fn parse_width(typed: &str) -> Option<u32> {
    let digits = typed.trim().trim_end_matches("px").trim();
    if digits.is_empty() || digits.len() > 5 || !digits.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let width: u32 = digits.parse().ok()?;
    (MIN_WIDTH..=MAX_WIDTH).contains(&width).then_some(width)
}

/// An empty Excalidraw drawing, as a note keeps one.
pub const EMPTY_DRAWING: &str =
    "```excalidraw\n{\"type\":\"excalidraw\",\"version\":2,\"elements\":[]}\n```";

/// The fenced block a drawing becomes in a note: the Excalidraw scene on the
/// clipboard (or in a file), as the note keeps it.
pub fn drawing_block(scene: &str) -> Result<String, kanso_render::Failure> {
    Ok(format!(
        "```excalidraw\n{}\n```",
        kanso_render::excalidraw::to_note(scene)?
    ))
}

/// The edit putting `block` in `text` at `at`, on lines of its own with a
/// blank line before and after it: on the cursor's line when that is blank,
/// otherwise after it. The cursor goes after the block.
pub fn insert_block(text: &str, at: usize, block: &str) -> Edit {
    let line = line_at(text, at);
    let blank = text[line.clone()].trim().is_empty();
    let (range, before_text) = if blank {
        (line.clone(), &text[..line.start])
    } else {
        (line.end..line.end, &text[..line.end])
    };
    let after_text = &text[range.end..];
    let before = if before_text.is_empty() || before_text.ends_with("\n\n") {
        ""
    } else if before_text.ends_with('\n') {
        "\n"
    } else {
        "\n\n"
    };
    let after = if after_text.starts_with("\n\n") || (after_text.is_empty() && blank) {
        ""
    } else if after_text.starts_with('\n') || after_text.is_empty() {
        "\n"
    } else {
        "\n\n"
    };
    let new = format!("{before}{block}{after}");
    let caret = range.start + before.len() + block.len();
    Edit {
        range,
        text: new,
        selection: caret..caret,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn images_on_their_own_line() {
        assert_eq!(
            line_image("![Cat|300](inkdrop://file:a)"),
            Some(LineImage {
                src: "inkdrop://file:a".into(),
                alt: "Cat".into(),
                width: Some(300),
                bar: true,
            })
        );
        assert_eq!(
            line_image("  ![](https://example.com/a.png) "),
            Some(LineImage {
                src: "https://example.com/a.png".into(),
                alt: String::new(),
                width: None,
                bar: false,
            })
        );
        assert_eq!(
            line_image(r#"<img src="inkdrop://file:a" width="240">"#).map(|i| i.width),
            Some(Some(240))
        );
        assert_eq!(line_image("Text ![a](b)"), None);
        assert_eq!(line_image("![a](b) text"), None);
        assert_eq!(line_image("![a](b \"title\")"), None);
        assert_eq!(line_image("[a](b)"), None);
    }

    #[test]
    fn widths_keep_the_form_they_were_written_in() {
        let bar = line_image("![Cat|300](a.png)").unwrap();
        assert_eq!(written(&bar, Some(120)), "![Cat|120](a.png)");
        assert_eq!(written(&bar, None), "![Cat](a.png)");
        let plain = line_image("![Cat](a.png)").unwrap();
        assert_eq!(
            written(&plain, Some(240)),
            r#"<img src="a.png" alt="Cat" width="240">"#
        );
    }

    #[test]
    fn resizing_the_cursors_line() {
        let text = "Intro\n\n  ![Cat](a.png)\n\nEnd";
        let at = text.find("Cat").unwrap();
        let edit = resize_at(text, at, Some(240)).unwrap();
        assert_eq!(
            edit.apply(text),
            "Intro\n\n  <img src=\"a.png\" alt=\"Cat\" width=\"240\">\n\nEnd"
        );
        assert_eq!(resize_at(text, 0, Some(240)), None);
    }

    #[test]
    fn typed_widths() {
        assert_eq!(parse_width("320"), Some(320));
        assert_eq!(parse_width(" 320px "), Some(320));
        assert_eq!(parse_width("12"), None);
        assert_eq!(parse_width("99999"), None);
        assert_eq!(parse_width("wide"), None);
        assert_eq!(parse_width(""), None);
    }

    #[test]
    fn drawings_go_on_lines_of_their_own() {
        let edit = insert_block("Before", 3, "B");
        assert_eq!(edit.apply("Before"), "Before\n\nB\n");
        let edit = insert_block("One\n\n\nTwo", 5, "B");
        assert_eq!(edit.apply("One\n\n\nTwo"), "One\n\nB\n\nTwo");
        let edit = insert_block("", 0, "B");
        assert_eq!(edit.apply(""), "B");
        assert!(kanso_render::excalidraw::is_scene(
            EMPTY_DRAWING
                .trim_start_matches("```excalidraw\n")
                .trim_end_matches("\n```")
        ));
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: None,
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        /// Round trip: an image written at a width reads back as that
        /// image at that width.
        #[test]
        fn prop_written_images_read_back(
            alt in "[A-Za-z0-9 ]{0,12}",
            src in "(inkdrop://file:[a-z0-9]{1,8}|https://example\\.com/[a-z]{1,8}\\.png)",
            width in prop::option::of(MIN_WIDTH..MAX_WIDTH),
            bar in any::<bool>(),
        ) {
            let alt = alt.trim().to_owned();
            let image = LineImage { src: src.clone(), alt: alt.clone(), width: None, bar };
            let back = line_image(&written(&image, width)).unwrap();
            prop_assert_eq!(back.src, src);
            prop_assert_eq!(back.alt, alt);
            prop_assert_eq!(back.width, width);
        }

        /// Invariants: a dragged width is whole, at least the minimum, and
        /// at most the room there is (or the minimum, when that is less).
        #[test]
        fn prop_drag_widths_stay_in_bounds(x in -500f32..3000., left in 0f32..800., max in 0f32..2000.) {
            let width = drag_width(x, left, max);
            prop_assert!(width >= MIN_WIDTH);
            prop_assert!(width as f32 <= max.max(MIN_WIDTH as f32).round());
        }

        /// Round trip: a scene put into a note as a block is a fenced
        /// drawing whose elements are the scene's.
        #[test]
        fn prop_drawing_blocks_round_trip(n in 0usize..5, seed in any::<i32>()) {
            let elements: Vec<serde_json::Value> = (0..n)
                .map(|i| serde_json::json!({"type": "rectangle", "x": i * 10, "y": 0, "width": 5, "height": 5, "seed": seed}))
                .collect();
            let scene = serde_json::json!({"type": "excalidraw/clipboard", "elements": elements}).to_string();
            let block = drawing_block(&scene).unwrap();
            let source = block
                .strip_prefix("```excalidraw\n")
                .and_then(|rest| rest.strip_suffix("\n```"))
                .unwrap();
            prop_assert!(kanso_render::excalidraw::is_scene(source));
            let back: serde_json::Value = serde_json::from_str(source).unwrap();
            prop_assert_eq!(back["elements"].as_array().unwrap().len(), n);
        }

        /// Invariants: a block goes in whole, on lines of its own, and the
        /// rest of the text is kept.
        #[test]
        fn prop_blocks_go_in_on_their_own_lines(text in "[a-z\n]{0,30}", at in 0usize..40) {
            let at = at.min(text.len());
            let edit = insert_block(&text, at, "BLOCK");
            let after = edit.apply(&text);
            prop_assert!(after.lines().any(|line| line == "BLOCK"));
            prop_assert_eq!(after.replacen("BLOCK", "", 1).replace('\n', ""), text.replace('\n', ""));
            prop_assert_eq!(&after[edit.selection.start - 5..edit.selection.start], "BLOCK");
        }
    }
}
