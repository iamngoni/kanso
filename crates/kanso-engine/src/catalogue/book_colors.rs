//! The color a notebook can show, so it's easy to find in the sidebar: kept
//! in the notebook as `#rrggbb` ([`kanso_core::model::BOOK_COLOR_FIELD`]) so
//! it syncs. The swatches offered are the tag colors; any other color is
//! typed as hex or picked. A color is `0xRRGGBB`.

use kanso_core::model::BOOK_COLOR_FIELD;
use kanso_core::{Book, TagColor};

/// The color `book` keeps, as `0xRRGGBB`, if it keeps one. Pure.
pub fn book_color(book: &Book) -> Option<u32> {
    parse_hex(book.extra.get(BOOK_COLOR_FIELD)?.as_str()?)
}

/// `text` as a color: `#rrggbb` or `#rgb`, with or without the `#`, any
/// case, spaces around it ignored. Pure.
pub fn parse_hex(text: &str) -> Option<u32> {
    let text = text.trim();
    let digits = text.strip_prefix('#').unwrap_or(text);
    if !digits.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let value = u32::from_str_radix(digits, 16).ok()?;
    match digits.len() {
        6 => Some(value),
        // Each digit doubled: `#fa0` is `#ffaa00`.
        3 => {
            let (r, g, b) = ((value >> 8) & 0xf, (value >> 4) & 0xf, value & 0xf);
            Some(((r * 0x11) << 16) | ((g * 0x11) << 8) | (b * 0x11))
        }
        _ => None,
    }
}

/// `rgb` as the notebook keeps it: `#rrggbb`. Pure.
pub fn hex(rgb: u32) -> String {
    format!("#{:06x}", rgb & 0xff_ffff)
}

/// The swatches the grid offers: the tag colors, after "No color". Pure.
pub fn presets() -> Vec<TagColor> {
    TagColor::ALL
        .into_iter()
        .filter(|color| *color != TagColor::Default)
        .collect()
}

/// The color a swatch shows, as `0xRRGGBB`: the same on every device. Pure.
pub fn preset_rgb(color: TagColor) -> u32 {
    color.rgb()
}

#[cfg(test)]
mod tests {
    use kanso_core::Book;
    use proptest::prelude::*;
    use proptest::test_runner::FileFailurePersistence;
    use serde_json::json;

    use super::*;

    #[test]
    fn hex_is_read_in_its_usual_spellings() {
        assert_eq!(parse_hex("#f8f8f8"), Some(0xf8f8f8));
        assert_eq!(parse_hex("F8F8F8"), Some(0xf8f8f8));
        assert_eq!(parse_hex("  #2185D0 "), Some(0x2185d0));
        assert_eq!(parse_hex("#fa0"), Some(0xffaa00));
        assert_eq!(parse_hex("000"), Some(0));
        for text in [
            "", "#", "#ff", "#ffff", "#fffffff", "#ggg", "+fff", "#-ff", "blue", "# fff",
        ] {
            assert_eq!(parse_hex(text), None, "{text:?}");
        }
        assert_eq!(hex(0xf8f8f8), "#f8f8f8");
        assert_eq!(hex(0x00000a), "#00000a");
    }

    #[test]
    fn a_notebook_keeps_its_color_or_none() {
        let colored = |value: serde_json::Value| {
            let book: Book =
                serde_json::from_value(json!({"_id": "book:a", "name": "A", "kansoColor": value}))
                    .unwrap();
            book_color(&book)
        };
        assert_eq!(colored(json!("#2185d0")), Some(0x2185d0));
        assert_eq!(colored(json!("#FFF")), Some(0xffffff));
        for value in [json!("blue"), json!("#12"), json!(3), json!(null)] {
            assert_eq!(colored(value.clone()), None, "{value}");
        }
        let plain: Book = serde_json::from_value(json!({"_id": "book:a", "name": "A"})).unwrap();
        assert_eq!(book_color(&plain), None);
    }

    #[test]
    fn the_swatches_are_the_tag_colors_after_no_color() {
        let presets = presets();
        assert_eq!(presets.len(), TagColor::ALL.len() - 1);
        assert!(!presets.contains(&TagColor::Default));
        assert_eq!(presets[0], TagColor::Red);
        assert_eq!(preset_rgb(TagColor::Blue), 0x2185d0);
        // Every swatch can be kept and read back.
        for color in presets {
            assert_eq!(parse_hex(&hex(preset_rgb(color))), Some(preset_rgb(color)));
        }
    }

    fn config() -> ProptestConfig {
        ProptestConfig {
            cases: 256,
            failure_persistence: Some(Box::new(FileFailurePersistence::WithSource("regressions"))),
            ..ProptestConfig::default()
        }
    }

    proptest! {
        #![proptest_config(config())]

        #[test]
        fn prop_a_kept_color_reads_back(rgb in 0u32..=0xff_ffff) {
            prop_assert_eq!(parse_hex(&hex(rgb)), Some(rgb));
            prop_assert_eq!(parse_hex(&hex(rgb).to_uppercase()), Some(rgb));
        }

        #[test]
        fn prop_only_hex_digits_are_read(text in "\\PC{0,8}") {
            if let Some(rgb) = parse_hex(&text) {
                prop_assert!(rgb <= 0xff_ffff);
                let digits = text.trim().trim_start_matches('#');
                prop_assert!(digits.chars().all(|c| c.is_ascii_hexdigit()));
            }
        }
    }
}
