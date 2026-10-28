//! Rich rendering for notes: TeX math (MathJax, run in QuickJS), Mermaid
//! diagrams and Excalidraw scenes, each drawn as a standalone SVG in the
//! colors of a theme; and a note as one self-contained HTML page.
//!
//! The preview asks for a drawing with [`request`] and shows it once
//! [`cached`] has it: drawing happens on a worker thread, never on the UI's.
//! An export, which waits anyway, calls [`render_blocking`]. Drawings are
//! kept in a bounded cache, most recently used first.
//!
//! Sizes are in points for text set at [`TEXT_SIZE`]; a caller showing text
//! at another size scales them.

mod cache;
pub mod excalidraw;
pub mod html;
mod js;
pub mod markdown_safe;
mod math;
mod mermaid;
mod pele;
pub mod plugin;
mod rough;
pub mod svg;
pub mod typography;

use std::sync::Arc;

pub use cache::{cached, forget_plugin_drawings, render_blocking, request};
pub use mermaid::is_supported as mermaid_is_supported;
pub use mermaid::{
    Engine as MermaidEngine, engine as mermaid_engine, render_with as render_mermaid_with,
    set_engine as set_mermaid_engine,
};
pub use plugin::{
    MAX_PLUGIN_SVG_BYTES, PluginRenderer, is_plugin_language, plugin_languages,
    sanitize_plugin_svg, set_plugin_renderer,
};
pub use svg::set_fonts;

/// The text size drawings are measured for.
pub const TEXT_SIZE: f32 = 16.;

/// A color as `0xRRGGBB`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Rgb(pub u32);

impl Rgb {
    /// `#rrggbb`.
    pub fn hex(self) -> String {
        format!("#{:06x}", self.0 & 0xFF_FFFF)
    }

    fn channels(self) -> [f32; 3] {
        [16, 8, 0].map(|shift| ((self.0 >> shift) & 0xFF) as f32)
    }

    fn from_channels(c: [f32; 3]) -> Self {
        let byte = |v: f32| v.round().clamp(0., 255.) as u32;
        Rgb((byte(c[0]) << 16) | (byte(c[1]) << 8) | byte(c[2]))
    }

    /// `self` laid over `under` at `alpha` (0 to 1).
    pub fn over(self, under: Rgb, alpha: f32) -> Rgb {
        let alpha = alpha.clamp(0., 1.);
        let (a, b) = (self.channels(), under.channels());
        Rgb::from_channels([0, 1, 2].map(|i| a[i] * alpha + b[i] * (1. - alpha)))
    }
}

/// The colors a drawing is made in: the preview theme's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Palette {
    /// Whether the page is dark: Excalidraw scenes are drawn for it as
    /// Excalidraw's own dark mode draws them.
    pub dark: bool,
    /// The page.
    pub background: Rgb,
    pub text: Rgb,
    /// Secondary text: captions, lines in diagrams.
    pub muted: Rgb,
    /// Links and highlights.
    pub accent: Rgb,
    pub border: Rgb,
    /// Code blocks and other quiet fills.
    pub surface: Rgb,
    /// Headings, when the theme gives them a color of their own.
    pub heading: Option<Rgb>,
    /// Quoted text and the bar beside it.
    pub quote: Rgb,
    /// Finished tasks, and list markers that step back.
    pub task_done: Rgb,
    /// A code block's text, when the theme gives it one.
    pub code_text: Option<Rgb>,
    /// Inline code's fill, and its text when the theme gives it one.
    pub inline_code_background: Rgb,
    pub inline_code_text: Option<Rgb>,
    /// A table's header row, when the theme gives it a fill of its own.
    pub table_header_background: Option<Rgb>,
}

impl Palette {
    /// Kanso's paper theme.
    pub const LIGHT: Palette = Palette {
        dark: false,
        background: Rgb(0xFBFAF7),
        text: Rgb(0x2B2A27),
        muted: Rgb(0x6F6B64),
        accent: Rgb(0x8A6A3D),
        border: Rgb(0xE4E0D8),
        surface: Rgb(0xF3F1EC),
        heading: None,
        quote: Rgb(0x6F6B64),
        task_done: Rgb(0x6F6B64),
        code_text: None,
        inline_code_background: Rgb(0xF3F1EC),
        inline_code_text: None,
        table_header_background: None,
    };

    /// Kanso's graphite theme.
    pub const DARK: Palette = Palette {
        dark: true,
        background: Rgb(0x1E1E1C),
        text: Rgb(0xE6E3DD),
        muted: Rgb(0x9C978E),
        accent: Rgb(0xD2AE78),
        border: Rgb(0x3A3935),
        surface: Rgb(0x282826),
        heading: None,
        quote: Rgb(0x9C978E),
        task_done: Rgb(0x9C978E),
        code_text: None,
        inline_code_background: Rgb(0x282826),
        inline_code_text: None,
        table_header_background: None,
    };
}

/// What a source is.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum Kind {
    /// `$…$`: set in the line of text.
    InlineMath,
    /// `$$…$$` or a `math` fence: set on its own, centred.
    DisplayMath,
    /// A `mermaid` fence.
    Mermaid,
    /// An `excalidraw` fence, or an attached `.excalidraw` file: the
    /// scene's JSON.
    Excalidraw,
    /// A fence in a language a plugin draws; see [`Job::plugin`].
    Plugin,
}

/// One thing to draw.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Job {
    pub kind: Kind,
    pub source: Arc<str>,
    pub palette: Palette,
    /// What draws a Mermaid diagram: part of the job, so choosing another
    /// engine draws again rather than showing what the cache has.
    pub mermaid: MermaidEngine,
    /// For [`Kind::Plugin`]: the fence's language (lowercase). `None` for
    /// every other kind.
    pub language: Option<Arc<str>>,
}

impl Job {
    /// A job drawn with the Mermaid engine chosen in [`set_mermaid_engine`].
    pub fn new(kind: Kind, source: impl Into<Arc<str>>, palette: Palette) -> Self {
        Job {
            kind,
            source: source.into(),
            palette,
            mermaid: mermaid::engine(),
            language: None,
        }
    }
}

/// A finished drawing.
#[derive(Clone, Debug, PartialEq)]
pub struct Svg {
    /// A standalone SVG document, its size set in points.
    pub markup: String,
    pub width: f32,
    pub height: f32,
    /// For inline math: how far below the top the text's baseline falls.
    pub baseline: Option<f32>,
}

/// Why a source wasn't drawn. The preview shows its source instead, with
/// [`Failure::badge`] beside it.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Failure {
    /// A diagram of a kind Kanso doesn't draw, by its keyword.
    Unsupported(String),
    /// A mistake in the source.
    Invalid(String),
    /// A fence in a language no plugin draws (any more): the language.
    NoPlugin(String),
}

impl Failure {
    /// A few words for the badge beside the source.
    pub fn badge(&self) -> String {
        match self {
            Failure::Unsupported(kind) if kind.is_empty() => "Not a diagram Kanso draws".into(),
            Failure::Unsupported(kind) => format!("{kind} diagrams aren't drawn yet"),
            Failure::Invalid(_) => "Couldn't draw this".into(),
            Failure::NoPlugin(language) => format!("No plugin draws {language} blocks"),
        }
    }

    /// The details, for a tooltip.
    pub fn detail(&self) -> &str {
        match self {
            Failure::Unsupported(_) | Failure::NoPlugin(_) => "",
            Failure::Invalid(message) => message,
        }
    }
}

/// A drawing, or why there isn't one.
pub type Outcome = Result<Svg, Failure>;

/// Draws `job` on this thread (math waits for MathJax's thread). Prefer
/// [`render_blocking`], which caches.
pub fn render(job: &Job) -> Outcome {
    match job.kind {
        Kind::InlineMath => math::render(&job.source, false, &job.palette),
        Kind::DisplayMath => math::render(&job.source, true, &job.palette),
        Kind::Mermaid => mermaid::render_with(job.mermaid, &job.source, &job.palette),
        Kind::Excalidraw => excalidraw::render(&job.source, &job.palette),
        Kind::Plugin => plugin::render(job),
    }
}

/// Whether `$…$` in prose was meant as math, by Pandoc's rule: the formula
/// doesn't start or end with a space, and a single `$` closing it isn't
/// followed by a digit. So "$5 and $10" stays prose. `source` is the span
/// with its dollar signs; `next` the character after it. Pure.
pub fn is_inline_math(source: &str, next: Option<char>) -> bool {
    let opening = source.len() - source.trim_start_matches('$').len();
    let inner = source.trim_start_matches('$').trim_end_matches('$');
    if inner.is_empty() || opening == 0 {
        return false;
    }
    let starts_clean = !inner.starts_with(char::is_whitespace);
    let ends_clean = !inner.ends_with(char::is_whitespace);
    let digit_after = opening == 1 && next.is_some_and(|c| c.is_ascii_digit());
    starts_clean && ends_clean && !digit_after
}

/// Escapes text for XML and HTML content and attribute values.
pub(crate) fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(256))]

        /// Invariants: mixing stays between the two colors, channel by
        /// channel, and its ends are the colors themselves.
        #[test]
        fn prop_over_stays_between(a in any::<u32>(), b in any::<u32>(), alpha in 0f32..=1.) {
            let (a, b) = (Rgb(a & 0xFF_FFFF), Rgb(b & 0xFF_FFFF));
            let mixed = a.over(b, alpha).channels();
            for ((m, x), y) in mixed.iter().zip(a.channels()).zip(b.channels()) {
                prop_assert!(*m >= x.min(y) && *m <= x.max(y));
            }
            prop_assert_eq!(a.over(b, 1.), a);
            prop_assert_eq!(a.over(b, 0.), b);
        }

        /// Invariant: escaped text has no markup characters left, and
        /// unescaping gives it back.
        #[test]
        fn prop_escape_round_trips(text in "\\PC{0,40}") {
            let escaped = escape(&text);
            prop_assert!(!escaped.contains(['<', '>', '"', '\'']));
            let back = escaped
                .replace("&lt;", "<")
                .replace("&gt;", ">")
                .replace("&quot;", "\"")
                .replace("&#39;", "'")
                .replace("&amp;", "&");
            prop_assert_eq!(back, text);
        }
    }

    #[test]
    fn dollar_amounts_are_not_math() {
        assert!(is_inline_math("$x^2$", Some(' ')));
        assert!(is_inline_math("$$x$$", Some('1')));
        assert!(!is_inline_math("$5 and $", Some('1')));
        assert!(!is_inline_math("$ x$", None));
        assert!(!is_inline_math("$x$", Some('3')));
        assert!(!is_inline_math("$$", None));
    }

    proptest! {
        /// Invariant: a formula padded with a space inside its dollars is
        /// never taken for math.
        #[test]
        fn prop_padded_dollars_are_prose(inner in "[a-z0-9+]{1,10}", next in prop::option::of(any::<char>())) {
            let (before, after, tight) = (format!("$ {inner}$"), format!("${inner} $"), format!("${inner}$"));
            prop_assert!(!is_inline_math(&before, next));
            prop_assert!(!is_inline_math(&after, next));
            prop_assert_eq!(is_inline_math(&tight, next), !next.is_some_and(|c| c.is_ascii_digit()));
        }
    }

    #[test]
    fn hex_examples() {
        assert_eq!(Rgb(0xABCDEF).hex(), "#abcdef");
        assert_eq!(Rgb(0x0000FF).hex(), "#0000ff");
    }
}
