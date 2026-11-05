//! Small SVG tools: reading and rewriting a document's root tag, and
//! flattening a drawing (text to outlines, in the fonts this machine has)
//! cropped to what it really covers.

use std::hash::{Hash as _, Hasher as _};
use std::sync::{Arc, LazyLock, OnceLock};

/// A number as SVG writes it: at most two decimals, no trailing zeros.
pub fn number(value: f32) -> String {
    let rounded = (value * 100.).round() / 100.;
    let text = format!("{rounded:.2}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" {
        "0".into()
    } else {
        text.into()
    }
}

/// The attributes of a document's root `<svg>` tag, to read or rewrite.
#[derive(Clone, Debug, PartialEq)]
pub struct RootTag {
    /// Where the tag sits in the document.
    span: std::ops::Range<usize>,
    /// Its attributes in order, values as written (still escaped).
    attributes: Vec<(String, String)>,
    self_closing: bool,
}

impl RootTag {
    /// The first `<svg …>` tag of `svg`.
    pub fn parse(svg: &str) -> Option<RootTag> {
        let start = svg.find("<svg")?;
        let bytes = svg.as_bytes();
        let mut at = start + 4;
        if !bytes
            .get(at)
            .is_some_and(|b| b.is_ascii_whitespace() || *b == b'>' || *b == b'/')
        {
            return None;
        }
        let mut attributes = Vec::new();
        loop {
            while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
                at += 1;
            }
            match bytes.get(at)? {
                b'>' => {
                    return Some(RootTag {
                        span: start..at + 1,
                        attributes,
                        self_closing: false,
                    });
                }
                b'/' if bytes.get(at + 1) == Some(&b'>') => {
                    return Some(RootTag {
                        span: start..at + 2,
                        attributes,
                        self_closing: true,
                    });
                }
                _ => {}
            }
            let name_start = at;
            while bytes
                .get(at)
                .is_some_and(|b| !b.is_ascii_whitespace() && !matches!(b, b'=' | b'>' | b'/'))
            {
                at += 1;
            }
            let name = svg.get(name_start..at)?.to_owned();
            if name.is_empty() {
                return None;
            }
            while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
                at += 1;
            }
            if bytes.get(at) != Some(&b'=') {
                attributes.push((name, String::new()));
                continue;
            }
            at += 1;
            while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
                at += 1;
            }
            let quote = *bytes.get(at)?;
            if quote != b'"' && quote != b'\'' {
                return None;
            }
            let value_start = at + 1;
            let value_end = value_start + svg.get(value_start..)?.find(quote as char)?;
            attributes.push((name, svg[value_start..value_end].to_owned()));
            at = value_end + 1;
        }
    }

    /// An attribute's value, as written.
    pub fn get(&self, name: &str) -> Option<&str> {
        self.attributes
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    /// The `viewBox`, as `[x, y, width, height]`.
    pub fn view_box(&self) -> Option<[f32; 4]> {
        let values: Vec<f32> = self
            .get("viewBox")?
            .split([' ', ','])
            .filter(|s| !s.is_empty())
            .map(str::parse)
            .collect::<Result<_, _>>()
            .ok()?;
        let [x, y, w, h] = values.try_into().ok()?;
        Some([x, y, w, h])
    }

    /// Sets an attribute (given unescaped-safe text: numbers, keywords).
    pub fn with(mut self, name: &str, value: String) -> Self {
        match self.attributes.iter_mut().find(|(n, _)| n == name) {
            Some((_, v)) => *v = value,
            None => self.attributes.push((name.to_owned(), value)),
        }
        self
    }

    pub fn without(mut self, name: &str) -> Self {
        self.attributes.retain(|(n, _)| n != name);
        self
    }

    /// `svg` with this tag in place of its root tag.
    pub fn write(&self, svg: &str) -> String {
        let mut tag = String::from("<svg");
        for (name, value) in &self.attributes {
            tag.push(' ');
            tag.push_str(name);
            tag.push_str("=\"");
            tag.push_str(&value.replace('"', "&quot;"));
            tag.push('"');
        }
        tag.push_str(if self.self_closing { "/>" } else { ">" });
        format!("{}{tag}{}", &svg[..self.span.start], &svg[self.span.end..])
    }
}

/// The fonts the app hands over with [`set_fonts`], read once when the font
/// database is built.
static EXTRA_FONTS: OnceLock<Vec<Vec<u8>>> = OnceLock::new();

/// The database [`fonts`] hands out, built on first use.
static DATABASE: LazyLock<Arc<usvg::fontdb::Database>> = LazyLock::new(|| {
    Arc::new(font_database(
        EXTRA_FONTS.get().map_or(&[][..], Vec::as_slice),
    ))
});

/// Adds the app's own fonts (the bytes of TrueType and OpenType files) to
/// the ones drawings are set in, so a diagram's text is set in the faces the
/// rest of the app uses and not in whichever the phone has. Faces that don't
/// parse are skipped. It takes effect only if it is called before the first
/// drawing, and only once: it says whether the fonts were taken.
// no-prop-test: hands bytes to a process-wide cell; `font_database` is tested.
pub fn set_fonts(fonts: Vec<Vec<u8>>) -> bool {
    LazyLock::get(&DATABASE).is_none() && EXTRA_FONTS.set(fonts).is_ok()
}

/// The fonts drawings are set in: this machine's. The font database finds
/// none of a phone's on its own (it has no folder for Android, and looks for
/// iOS's where Linux keeps them), so those are loaded from where the system
/// keeps them, and the generic families are set to faces that exist there.
/// Without that, text in a drawing would be left out, or set in whichever
/// face came first. The app's own fonts come from [`set_fonts`].
// no-prop-test: reads the machine's fonts; the drawing tests go through it.
pub(crate) fn fonts() -> Arc<usvg::fontdb::Database> {
    DATABASE.clone()
}

/// A font database of this machine's fonts and then `extra`, the app's.
// no-prop-test: reads the machine's fonts; the test below checks what it adds.
fn font_database(extra: &[Vec<u8>]) -> usvg::fontdb::Database {
    let mut db = usvg::fontdb::Database::new();
    db.load_system_fonts();
    #[cfg(target_os = "ios")]
    {
        db.load_fonts_dir("/System/Library/Fonts");
        db.set_serif_family("Times New Roman");
        db.set_sans_serif_family("Helvetica");
        db.set_monospace_family("Courier");
    }
    #[cfg(target_os = "android")]
    {
        db.load_fonts_dir("/system/fonts");
        db.set_serif_family("Noto Serif");
        db.set_sans_serif_family("Roboto");
        db.set_monospace_family("Droid Sans Mono");
    }
    for bytes in extra {
        db.load_font_data(bytes.clone());
    }
    db
}

/// A flattened drawing: standalone SVG and its size in points.
#[derive(Clone, Debug, PartialEq)]
pub struct Flat {
    pub markup: String,
    pub width: f32,
    pub height: f32,
}

/// How a drawing is parsed: with this machine's fonts, and a family it
/// lacks falling back to its sans-serif, then to any face, rather than
/// leaving the text out.
fn tree_options() -> usvg::Options<'static> {
    let default_font = usvg::FontResolver::default_font_selector();
    usvg::Options {
        fontdb: fonts(),
        font_resolver: usvg::FontResolver {
            select_font: Box::new(move |font, db| {
                default_font(font, db).or_else(|| {
                    let sans = usvg::fontdb::Query {
                        families: &[usvg::fontdb::Family::SansSerif],
                        ..Default::default()
                    };
                    db.query(&sans).or_else(|| db.faces().next().map(|f| f.id))
                })
            }),
            select_fallback: usvg::FontResolver::default_fallback_selector(),
        },
        ..usvg::Options::default()
    }
}

/// `svg` drawn as a PNG, `scale` pixels to the point, with the long side cut
/// to at most `max_side` pixels (the drawing is made smaller to fit, not
/// cropped) and the background left clear. `None` if it doesn't parse or has
/// no size.
// no-prop-test: hands the drawing to resvg; the canvas export's tests draw through it.
pub fn rasterize(svg: &str, scale: f32, max_side: u32) -> Option<Vec<u8>> {
    let tree = usvg::Tree::from_str(svg, &tree_options()).ok()?;
    let size = tree.size();
    let (w, h) = (size.width() * scale, size.height() * scale);
    if !(w.is_finite() && h.is_finite() && w > 0. && h > 0.) {
        return None;
    }
    let fit = (max_side as f32 / w.max(h)).min(1.);
    let px_w = ((w * fit).round() as u32).max(1);
    let px_h = ((h * fit).round() as u32).max(1);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(px_w, px_h)?;
    let transform = resvg::tiny_skia::Transform::from_scale(
        px_w as f32 / size.width(),
        px_h as f32 / size.height(),
    );
    resvg::render(&tree, transform, &mut pixmap.as_mut());
    pixmap.encode_png().ok()
}

/// `svg` with its text turned to outlines (so it looks the same wherever it
/// is shown, fonts or not), cropped to what it draws plus `pad` on each
/// side, and sized in points. `None` if it doesn't parse or draws nothing.
pub fn flatten(svg: &str, pad: f32) -> Option<Flat> {
    let tree = usvg::Tree::from_str(svg, &tree_options()).ok()?;
    let bounds = tree.root().abs_layer_bounding_box();
    if !tree.root().has_children() {
        return None;
    }
    let (x, y) = (bounds.x() - pad, bounds.y() - pad);
    let (width, height) = (bounds.width() + 2. * pad, bounds.height() + 2. * pad);
    if !(width.is_finite() && height.is_finite()) || width > 20_000. || height > 20_000. {
        return None;
    }
    // Several drawings can share a page (an export): their IDs mustn't.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    svg.hash(&mut hasher);
    let written = tree.to_string(&usvg::WriteOptions {
        id_prefix: Some(format!("k{:x}-", hasher.finish() & 0xFFFF_FFFF)),
        coordinates_precision: 3,
        transforms_precision: 5,
        indent: usvg::Indent::None,
        attributes_indent: usvg::Indent::None,
        ..usvg::WriteOptions::default()
    });
    let markup = RootTag::parse(&written)?
        .with("width", number(width))
        .with("height", number(height))
        .with(
            "viewBox",
            format!(
                "{} {} {} {}",
                number(x),
                number(y),
                number(width),
                number(height)
            ),
        )
        .write(&written);
    Some(Flat {
        markup,
        width,
        height,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    #[test]
    fn root_tags_read_and_rewrite() {
        let svg = r#"<?xml version="1.0"?><svg width="100%" viewBox="0 -10 20 30" style='a: b'><rect/></svg>"#;
        let root = RootTag::parse(svg).unwrap();
        assert_eq!(root.get("width"), Some("100%"));
        assert_eq!(root.get("style"), Some("a: b"));
        assert_eq!(root.view_box(), Some([0., -10., 20., 30.]));
        let out = root
            .with("width", "20".into())
            .with("height", "30".into())
            .without("style")
            .write(svg);
        assert_eq!(
            out,
            r#"<?xml version="1.0"?><svg width="20" viewBox="0 -10 20 30" height="30"><rect/></svg>"#
        );
        assert_eq!(RootTag::parse("<svgx>"), None);
        assert_eq!(RootTag::parse("<svg a=b>"), None);
    }

    #[test]
    fn flattening_crops_to_what_is_drawn() {
        // A rectangle partly above the view box: Mermaid's subgraph frames
        // can start there.
        let svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="100" height="50" viewBox="0 0 100 50"><rect x="10" y="-6" width="50" height="40" fill="#f00"/></svg>"##;
        let flat = flatten(svg, 2.).unwrap();
        assert_eq!((flat.width, flat.height), (54., 44.));
        let tree = usvg::Tree::from_str(&flat.markup, &usvg::Options::default()).unwrap();
        assert_eq!(tree.size().width(), 54.);
        assert!(flatten("<svg xmlns=\"http://www.w3.org/2000/svg\"/>", 0.).is_none());
        assert!(flatten("not svg", 0.).is_none());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Round trip: a number written and read back is within rounding of
        /// what it was.
        #[test]
        fn prop_numbers_round_trip(value in -10_000f32..10_000.) {
            let back: f32 = number(value).parse().unwrap();
            prop_assert!((back - value).abs() <= 0.005 + value.abs() * 1e-6);
        }

        /// Round trip: rewriting a root tag with the attributes it already
        /// has changes nothing a parser sees.
        #[test]
        fn prop_root_rewrite_is_stable(attrs in prop::collection::vec(("[a-z]{1,6}", "[a-z0-9 .%-]{0,8}"), 0..6)) {
            let mut seen = std::collections::HashSet::new();
            let attrs: Vec<_> = attrs.into_iter().filter(|(n, _)| seen.insert(n.clone())).collect();
            let body: String = attrs.iter().map(|(n, v)| format!(" {n}=\"{v}\"")).collect();
            let svg = format!("<svg{body}><g/></svg>");
            let root = RootTag::parse(&svg).unwrap();
            let again = root.write(&svg);
            prop_assert_eq!(&again, &svg);
            let reparsed = RootTag::parse(&again).unwrap();
            for (n, v) in &attrs {
                prop_assert_eq!(reparsed.get(n), Some(v.as_str()));
            }
        }
    }

    /// The app's mono face, as the phones ship it.
    const APP_FONT: &[u8] =
        include_bytes!("../../../mobile/assets/fonts/JetBrainsMono-Regular.ttf");

    // no-prop-test: one font file and one that isn't; there is nothing to vary.
    #[test]
    fn the_apps_fonts_join_the_machines_and_broken_ones_are_skipped() {
        let base = font_database(&[]);
        let with = font_database(&[APP_FONT.to_vec(), b"not a font".to_vec()]);
        assert_eq!(with.len(), base.len() + 1);
        assert!(with.faces().any(|face| {
            face.families
                .iter()
                .any(|(name, _)| name == "JetBrains Mono")
        }));
    }
}
