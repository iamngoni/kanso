//! Fenced blocks drawn by plugins.
//!
//! A plugin registers a language such as `chart`; a fence in that language
//! is handed to the plugin's renderer, which answers with SVG text. The
//! app sets the renderer with [`set_plugin_renderer`]; it runs on the
//! render workers like any other drawing, never on the UI's thread.
//!
//! The plugin's SVG is untrusted. [`sanitize_plugin_svg`] refuses what is
//! too big or isn't SVG, parses the rest with `usvg` (which runs no script
//! and loads nothing), writes the parsed drawing back out, and then checks
//! the result once more for anything that could reach out or run. What
//! comes back is a drawing of shapes, paths and gradients, and nothing else.

use std::collections::BTreeSet;
use std::hash::{Hash as _, Hasher as _};
use std::panic::AssertUnwindSafe;
#[cfg(test)]
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, LazyLock, RwLock};

use crate::{Failure, Job, Kind, Outcome, Palette, Svg, cache};

/// The most SVG text a plugin may return.
pub const MAX_PLUGIN_SVG_BYTES: usize = 512 * 1024;
/// The most a drawing may measure, either way, in points.
pub const MAX_PLUGIN_SVG_SIZE: f32 = 8192.;

/// What draws a language's blocks: the language, the block's source and the
/// theme's colors in; SVG text, or why there is none, out.
pub type PluginRenderer =
    dyn Fn(&str, &str, &Palette) -> Result<String, String> + Send + Sync + 'static;

#[derive(Default)]
struct Registry {
    renderer: Option<Arc<PluginRenderer>>,
    languages: BTreeSet<String>,
}

static REGISTRY: LazyLock<RwLock<Registry>> = LazyLock::new(Default::default);

/// Bumped each time the renderer or languages change, so a drawing made for
/// the old ones can be told from one made for the new.
static GENERATION: AtomicU64 = AtomicU64::new(0);

/// Moves on to a new generation: what is being drawn for the old renderer
/// is stale.
pub(crate) fn next_generation() {
    GENERATION.fetch_add(1, Ordering::SeqCst);
}

/// Which renderer and languages drawings are being made for.
pub(crate) fn generation() -> u64 {
    GENERATION.load(Ordering::SeqCst)
}

/// A language as it is compared: trimmed, lowercase.
fn normal(language: &str) -> String {
    language.trim().to_ascii_lowercase()
}

/// Sets what draws plugin blocks, and the languages it draws (`None` or an
/// empty set: none). Every kept plugin drawing is dropped, and a drawing
/// still being made for the old renderer isn't kept when it finishes.
pub fn set_plugin_renderer(renderer: Option<Arc<PluginRenderer>>, languages: BTreeSet<String>) {
    let languages: BTreeSet<String> = languages
        .iter()
        .map(|l| normal(l))
        .filter(|l| !l.is_empty())
        .collect();
    if let Ok(mut registry) = REGISTRY.write() {
        registry.renderer = renderer;
        registry.languages = languages;
    }
    cache::forget_plugin_drawings();
}

/// The languages plugins draw now.
pub fn plugin_languages() -> BTreeSet<String> {
    REGISTRY
        .read()
        .map(|registry| registry.languages.clone())
        .unwrap_or_default()
}

/// Whether a plugin draws `lang` blocks (ignoring case and surrounding
/// space) and there is a renderer to do it.
pub fn is_plugin_language(lang: &str) -> bool {
    REGISTRY.read().is_ok_and(|registry| {
        registry.renderer.is_some() && registry.languages.contains(&normal(lang))
    })
}

impl Job {
    /// A block of `language` for the plugin that draws it.
    pub fn plugin(
        language: impl Into<Arc<str>>,
        source: impl Into<Arc<str>>,
        palette: Palette,
    ) -> Job {
        let language: Arc<str> = language.into();
        let mut job = Job::new(Kind::Plugin, source, palette);
        job.language = Some(normal(&language).into());
        job
    }
}

/// Draws a plugin block: the renderer's SVG, cleaned.
pub(crate) fn render(job: &Job) -> Outcome {
    let language = job.language.as_deref().unwrap_or_default();
    let renderer = {
        let registry = REGISTRY
            .read()
            .map_err(|_| Failure::NoPlugin(language.into()))?;
        match &registry.renderer {
            Some(renderer) if registry.languages.contains(language) => renderer.clone(),
            _ => return Err(Failure::NoPlugin(language.into())),
        }
    };
    let drawn = std::panic::catch_unwind(AssertUnwindSafe(|| {
        renderer(language, &job.source, &job.palette)
    }))
    .map_err(|_| Failure::Invalid("The plugin stopped while drawing this.".into()))?
    .map_err(Failure::Invalid)?;
    let (markup, width, height) = sanitize_plugin_svg(&drawn).map_err(Failure::Invalid)?;
    Ok(Svg {
        markup,
        width,
        height,
        baseline: None,
        notices: Vec::new(),
    })
}

fn options() -> usvg::Options<'static> {
    let default_font = usvg::FontResolver::default_font_selector();
    usvg::Options {
        resources_dir: None,
        // No image is ever loaded, from a data URL or from anywhere else.
        image_href_resolver: usvg::ImageHrefResolver {
            resolve_data: Box::new(|_, _, _| None),
            resolve_string: Box::new(|_, _| None),
        },
        fontdb: crate::svg::fonts(),
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

/// Whether `text`'s first element is `<svg`, after any XML declaration,
/// comments and white space; and that it declares no DOCTYPE or entities.
fn has_svg_root(text: &str) -> bool {
    let lower = text.to_ascii_lowercase();
    if lower.contains("<!doctype") || lower.contains("<!entity") {
        return false;
    }
    let mut rest = text.trim_start_matches('\u{feff}').trim_start();
    loop {
        if let Some(after) = rest.strip_prefix("<?") {
            match after.find("?>") {
                Some(end) => rest = after[end + 2..].trim_start(),
                None => return false,
            }
        } else if let Some(after) = rest.strip_prefix("<!--") {
            match after.find("-->") {
                Some(end) => rest = after[end + 3..].trim_start(),
                None => return false,
            }
        } else {
            break;
        }
    }
    rest.strip_prefix("<svg")
        .and_then(|after| after.chars().next())
        .is_some_and(|c| c.is_whitespace() || c == '>' || c == '/')
}

/// Whether `svg` holds anything that could run or fetch. A last look at
/// what `usvg` wrote, in case it ever lets something through. Pure.
fn find_danger(svg: &str) -> Option<&'static str> {
    let lower = svg.to_ascii_lowercase();
    for (needle, why) in [
        ("<script", "a script"),
        ("<foreignobject", "an embedded page"),
        ("<image", "an image"),
        ("javascript:", "a script link"),
    ] {
        if lower.contains(needle) {
            return Some(why);
        }
    }
    let bytes = lower.as_bytes();
    let after_space = |mut at: usize| {
        while bytes.get(at).is_some_and(u8::is_ascii_whitespace) {
            at += 1;
        }
        at
    };
    for (at, _) in lower.match_indices("href") {
        let eq = after_space(at + 4);
        if bytes.get(eq) != Some(&b'=') {
            continue;
        }
        let mut value = after_space(eq + 1);
        if matches!(bytes.get(value), Some(b'"' | b'\'')) {
            value += 1;
        }
        if bytes.get(value) != Some(&b'#') {
            return Some("a link that leaves the drawing");
        }
    }
    for (at, _) in lower.match_indices("url(") {
        let mut value = after_space(at + 4);
        if matches!(bytes.get(value), Some(b'"' | b'\'')) {
            value += 1;
        }
        if bytes.get(value) != Some(&b'#') {
            return Some("a reference that leaves the drawing");
        }
    }
    for (at, _) in lower.match_indices("on") {
        let spaced = at > 0 && bytes[at - 1].is_ascii_whitespace();
        if !spaced {
            continue;
        }
        let mut end = at + 2;
        while bytes.get(end).is_some_and(u8::is_ascii_alphabetic) {
            end += 1;
        }
        if end > at + 2 && bytes.get(after_space(end)) == Some(&b'=') {
            return Some("an event handler");
        }
    }
    None
}

/// A plugin's SVG made safe to show: parsed, drawn from nothing but shapes,
/// paths, text outlines and gradients, and written out again. Returns it
/// with its size in points, or why it was refused. Pure.
pub fn sanitize_plugin_svg(text: &str) -> Result<(String, f32, f32), String> {
    if text.len() > MAX_PLUGIN_SVG_BYTES {
        return Err("The drawing is over 512 KB.".into());
    }
    if !has_svg_root(text) {
        return Err("The plugin didn't return an SVG drawing.".into());
    }
    let tree =
        std::panic::catch_unwind(AssertUnwindSafe(|| usvg::Tree::from_str(text, &options())))
            .map_err(|_| "The drawing couldn't be read.".to_string())?
            .map_err(|err| format!("The drawing couldn't be read: {err}."))?;
    let size = tree.size();
    let (width, height) = (size.width(), size.height());
    let fits = |v: f32| v.is_finite() && v > 0. && v <= MAX_PLUGIN_SVG_SIZE;
    if !fits(width) || !fits(height) {
        return Err("The drawing is empty or over 8192 points across.".into());
    }
    // Several drawings can share a page (an export): their IDs mustn't.
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut hasher);
    let written = tree.to_string(&usvg::WriteOptions {
        id_prefix: Some(format!("p{:x}-", hasher.finish() & 0xFFFF_FFFF)),
        ..usvg::WriteOptions::default()
    });
    if let Some(why) = find_danger(&written) {
        return Err(format!("The drawing was refused: it held {why}."));
    }
    Ok((written, width, height))
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    const SVG_OPEN: &str = r#"<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" width="200" height="100" viewBox="0 0 200 100">"#;

    fn chart(rects: usize, extra: &str) -> String {
        let mut svg = String::from(SVG_OPEN);
        for n in 0..rects {
            svg.push_str(&format!(
                r##"<rect x="{}" y="10" width="8" height="{}" fill="#336699"/>"##,
                n * 10,
                20 + n
            ));
        }
        svg.push_str(extra);
        svg.push_str("</svg>");
        svg
    }

    const PAYLOADS: &[&str] = &[
        "<script>alert(1)</script>",
        r#"<rect width="5" height="5" onload="alert(1)" onclick="x()"/>"#,
        r#"<image href="https://x/a.png" width="10" height="10"/>"#,
        r#"<image xlink:href="data:image/png;base64,AAAA" width="10" height="10"/>"#,
        r#"<foreignObject width="10" height="10"><div xmlns="http://www.w3.org/1999/xhtml">hi</div></foreignObject>"#,
        r#"<a href="javascript:alert(1)"><rect width="5" height="5"/></a>"#,
        r#"<a xlink:href="javascript:alert(1)"><text x="5" y="5">go</text></a>"#,
        r#"<rect width="5" height="5" style="background:url(https://x/y)"/>"#,
        r#"<rect width="5" height="5" style="fill:url(https://x/y)"/>"#,
        r##"<defs><rect id="a" width="5" height="5"/></defs><use href="https://x#a"/>"##,
        r##"<defs><rect id="a" width="5" height="5"/></defs><use xlink:href="https://x/y.svg#a"/>"##,
        r#"<style>rect{fill:url(https://x/y)}@import url(https://x/z.css);</style><rect width="5" height="5"/>"#,
    ];

    fn assert_clean(out: &str) {
        let lower = out.to_ascii_lowercase();
        for bad in [
            "<script",
            "<foreignobject",
            "<image",
            "javascript:",
            "https://",
            "http://x",
            "onload=",
            "onclick=",
            "data:image",
        ] {
            assert!(!lower.contains(bad), "{bad} survived in {out}");
        }
    }

    #[test]
    fn a_simple_chart_survives() {
        let svg = chart(
            5,
            r##"<text x="4" y="90" font-size="10" fill="#222">Totals</text>"##,
        );
        let (out, w, h) = sanitize_plugin_svg(&svg).unwrap();
        assert_eq!((w, h), (200., 100.));
        assert!(out.matches("<path").count() >= 5, "{out}");
        assert_clean(&out);
    }

    #[test]
    fn rects_are_kept_one_for_one() {
        let (out, ..) = sanitize_plugin_svg(&chart(7, "")).unwrap();
        assert_eq!(out.matches("<path").count(), 7, "{out}");
    }

    #[test]
    fn each_payload_is_gone() {
        let mut accepted = 0;
        for payload in PAYLOADS {
            let svg = chart(2, payload);
            if let Ok((out, ..)) = sanitize_plugin_svg(&svg) {
                assert_clean(&out);
                accepted += 1;
            }
        }
        // Cleaned, not just refused.
        assert!(accepted >= PAYLOADS.len() / 2, "{accepted}");
    }

    #[test]
    fn xml_declaration_and_comments_may_lead() {
        let svg = format!("\n<?xml version=\"1.0\"?>\n<!-- hi -->\n{}</svg>", SVG_OPEN);
        assert!(sanitize_plugin_svg(&svg).is_ok() || has_svg_root(&svg));
        assert!(has_svg_root(&svg));
    }

    #[test]
    fn what_is_not_an_svg_root_is_refused() {
        for text in [
            "",
            "hello",
            "<html><svg></svg></html>",
            "<svgx/>",
            "<!DOCTYPE svg [<!ENTITY a \"b\">]><svg xmlns=\"http://www.w3.org/2000/svg\"/>",
            "<?xml version=\"1.0\"?><!ENTITY a \"b\"><svg/>",
            "<?xml unclosed <svg/>",
        ] {
            assert!(sanitize_plugin_svg(text).is_err(), "{text}");
        }
    }

    #[test]
    fn big_input_and_big_drawings_are_refused() {
        let filler = " ".repeat(MAX_PLUGIN_SVG_BYTES);
        assert!(
            sanitize_plugin_svg(&format!(
                "{}{filler}</svg>",
                chart(1, "").trim_end_matches("</svg>")
            ))
            .is_err()
        );
        for (w, h) in [(9000, 10), (10, 8193), (0, 10), (10, 0)] {
            let svg = format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}"><rect width="5" height="5"/></svg>"#
            );
            assert!(sanitize_plugin_svg(&svg).is_err(), "{w}x{h}");
        }
        let svg = r#"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 20000 10"><rect width="5" height="5"/></svg>"#;
        assert!(sanitize_plugin_svg(svg).is_err());
    }

    #[test]
    fn the_last_check_catches_what_slips_past() {
        for bad in [
            "<svg><script/></svg>",
            "<svg><ForeignObject/></svg>",
            "<svg><image/></svg>",
            r#"<svg><a href="https://x"/></svg>"#,
            r#"<svg><use xlink:href = 'http://x#a'/></svg>"#,
            r#"<svg><rect fill="url(https://x)"/></svg>"#,
            r#"<svg><rect fill="url( 'x.svg#a')"/></svg>"#,
            r#"<svg onload="x()"></svg>"#,
            "<svg>\nOnClick = \"x\"</svg>",
            "<svg>javascript:x</svg>",
        ] {
            assert!(find_danger(bad).is_some(), "{bad}");
        }
        for fine in [
            r##"<svg><use href="#a"/><rect fill="url(#g)"/><text>on top, 100% done</text></svg>"##,
            r##"<svg><use xlink:href="#a"/><rect fill="url('#g')"/></svg>"##,
        ] {
            assert_eq!(find_danger(fine), None, "{fine}");
        }
    }

    /// The renderer is global: tests that set it take turns.
    static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

    fn languages(names: &[&str]) -> BTreeSet<String> {
        names.iter().map(|n| n.to_string()).collect()
    }

    fn fixed(svg: String) -> Option<Arc<PluginRenderer>> {
        Some(Arc::new(move |_: &str, _: &str, _: &Palette| {
            Ok(svg.clone())
        }))
    }

    #[test]
    fn no_renderer_means_no_plugin() {
        let _turn = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        set_plugin_renderer(None, languages(&["chart"]));
        assert!(!is_plugin_language("chart"));
        let job = Job::plugin("chart", "1 2 3", Palette::LIGHT);
        let outcome = crate::render_blocking(&job);
        let failure = outcome.as_ref().as_ref().unwrap_err();
        assert_eq!(failure, &Failure::NoPlugin("chart".into()));
        assert_eq!(failure.badge(), "No plugin draws chart blocks");
        assert_eq!(failure.detail(), "");
    }

    #[test]
    fn a_language_nobody_registered_is_no_plugin() {
        let _turn = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        set_plugin_renderer(fixed(chart(1, "")), languages(&["chart"]));
        assert!(is_plugin_language(" Chart "));
        assert!(!is_plugin_language("other"));
        let outcome = crate::render_blocking(&Job::plugin("other", "x", Palette::LIGHT));
        assert_eq!(
            outcome.as_ref().as_ref().unwrap_err(),
            &Failure::NoPlugin("other".into())
        );
        assert_eq!(plugin_languages(), languages(&["chart"]));
        set_plugin_renderer(None, BTreeSet::new());
    }

    #[test]
    fn a_renderer_that_returns_a_script_yields_a_clean_drawing() {
        let _turn = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        set_plugin_renderer(
            fixed(chart(
                3,
                "<script>alert(1)</script><image href=\"https://x/y.png\"/>",
            )),
            languages(&["chart"]),
        );
        let outcome = crate::render_blocking(&Job::plugin("chart", "script test", Palette::LIGHT));
        let svg = outcome.as_ref().as_ref().unwrap();
        assert!(!svg.markup.contains("<script"));
        assert!(!svg.markup.contains("<image"));
        assert_eq!(svg.markup.matches("<path").count(), 3);
        assert_eq!((svg.width, svg.height), (200., 100.));
        set_plugin_renderer(None, BTreeSet::new());
    }

    #[test]
    fn a_plugin_fence_is_drawn_into_the_page() {
        let _turn = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        set_plugin_renderer(fixed(chart(2, "<script>x</script>")), languages(&["chart"]));
        let page = crate::html::body(
            "# T\n\n```chart\n1 2\n```\n\n```other\nkeep\n```\n",
            &Palette::LIGHT,
            &|_| None,
        );
        assert!(page.contains("kanso-drawing"), "{page}");
        assert!(!page.contains("<script>x"));
        assert!(page.contains("keep"));
        set_plugin_renderer(None, BTreeSet::new());
    }

    #[test]
    fn a_renderer_that_fails_or_panics_shows_why() {
        let _turn = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let failing: Arc<PluginRenderer> = Arc::new(|_, source, _| match source {
            "panic" => panic!("boom"),
            _ => Err("bad data".into()),
        });
        set_plugin_renderer(Some(failing), languages(&["chart"]));
        for source in ["fail", "panic"] {
            let outcome = crate::render_blocking(&Job::plugin("chart", source, Palette::LIGHT));
            assert!(matches!(outcome.as_ref(), Err(Failure::Invalid(_))));
        }
        let outcome = crate::render_blocking(&Job::plugin("chart", "fail", Palette::LIGHT));
        assert_eq!(outcome.as_ref().as_ref().unwrap_err().detail(), "bad data");
        set_plugin_renderer(None, BTreeSet::new());
    }

    #[test]
    fn a_new_renderer_forgets_old_drawings() {
        let _turn = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        set_plugin_renderer(fixed(chart(1, "")), languages(&["chart"]));
        let job = Job::plugin("chart", "forget me", Palette::LIGHT);
        let first = crate::render_blocking(&job);
        assert!(crate::cached(&job).is_some());
        set_plugin_renderer(fixed(chart(4, "")), languages(&["chart"]));
        assert!(crate::cached(&job).is_none());
        let second = crate::render_blocking(&job);
        assert_eq!(
            first
                .as_ref()
                .as_ref()
                .unwrap()
                .markup
                .matches("<path")
                .count(),
            1
        );
        assert_eq!(
            second
                .as_ref()
                .as_ref()
                .unwrap()
                .markup
                .matches("<path")
                .count(),
            4
        );
        // Dropping the renderer drops the drawing too.
        set_plugin_renderer(None, BTreeSet::new());
        assert!(crate::cached(&job).is_none());
        // Other kinds are left alone.
        let math = Job::new(Kind::InlineMath, "x_plugin", Palette::LIGHT);
        crate::render_blocking(&math);
        crate::forget_plugin_drawings();
        assert!(crate::cached(&math).is_some());
    }

    #[test]
    fn a_drawing_in_flight_for_the_old_renderer_is_not_kept() {
        let _turn = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
        let (started_tx, started_rx) = std::sync::mpsc::channel::<()>();
        let (go_tx, go_rx) = std::sync::mpsc::channel::<()>();
        let go_rx = Mutex::new(go_rx);
        let started_tx = Mutex::new(started_tx);
        let slow: Arc<PluginRenderer> = Arc::new(move |_, _, _| {
            started_tx.lock().unwrap().send(()).ok();
            go_rx
                .lock()
                .unwrap()
                .recv_timeout(std::time::Duration::from_secs(5))
                .ok();
            Ok(chart(1, ""))
        });
        set_plugin_renderer(Some(slow), languages(&["chart"]));
        let job = Job::plugin("chart", "in flight", Palette::LIGHT);
        let (done_tx, done_rx) = std::sync::mpsc::channel();
        crate::request(job.clone(), move || {
            done_tx.send(()).ok();
        });
        started_rx.recv().unwrap();
        // Swap while the old renderer is still drawing.
        set_plugin_renderer(fixed(chart(6, "")), languages(&["chart"]));
        go_tx.send(()).unwrap();
        done_rx.recv().unwrap();
        // What landed is the new renderer's drawing, not the old one's.
        let kept = crate::cached(&job).unwrap();
        assert_eq!(
            kept.as_ref()
                .as_ref()
                .unwrap()
                .markup
                .matches("<path")
                .count(),
            6
        );
        set_plugin_renderer(None, BTreeSet::new());
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(64))]

        /// Invariant: whatever payloads are mixed into a chart, an accepted
        /// drawing has no script, page, image, outside link or handler.
        #[test]
        fn prop_payloads_never_survive(picks in prop::collection::vec(0..PAYLOADS.len(), 1..5), rects in 0usize..6) {
            let extra: String = picks.iter().map(|i| PAYLOADS[*i]).collect();
            if let Ok((out, ..)) = sanitize_plugin_svg(&chart(rects, &extra)) {
                assert_clean(&out);
                prop_assert!(find_danger(&out).is_none());
            }
        }

        /// Invariant: a valid chart keeps every rectangle it had.
        #[test]
        fn prop_rects_survive(rects in 1usize..30) {
            let (out, w, h) = sanitize_plugin_svg(&chart(rects, "")).unwrap();
            prop_assert_eq!(out.matches("<path").count(), rects);
            prop_assert_eq!((w, h), (200., 100.));
        }

        /// Invariant: a drawing wider or taller than 8192 points is never
        /// accepted, and one inside the limit is.
        #[test]
        fn prop_size_limit(w in 1u32..20_000, h in 1u32..20_000) {
            let svg = format!(
                r#"<svg xmlns="http://www.w3.org/2000/svg" width="{w}" height="{h}"><rect width="1" height="1"/></svg>"#
            );
            let fits = w <= 8192 && h <= 8192;
            prop_assert_eq!(sanitize_plugin_svg(&svg).is_ok(), fits);
        }

        /// Invariant: random bytes, and random text after an SVG start, are
        /// refused or cleaned, never a panic.
        #[test]
        fn prop_garbage_never_panics(bytes in prop::collection::vec(any::<u8>(), 0..400), tail in "\\PC{0,200}") {
            let text = String::from_utf8_lossy(&bytes).into_owned();
            let _ = sanitize_plugin_svg(&text);
            let _ = sanitize_plugin_svg(&format!("{SVG_OPEN}{tail}</svg>"));
        }

        /// Invariant: input over the byte limit is always refused.
        #[test]
        fn prop_oversize_refused(extra in 1usize..2000) {
            let svg = format!("{SVG_OPEN}{}</svg>", " ".repeat(MAX_PLUGIN_SVG_BYTES + extra));
            prop_assert!(sanitize_plugin_svg(&svg).is_err());
        }
    }
}
