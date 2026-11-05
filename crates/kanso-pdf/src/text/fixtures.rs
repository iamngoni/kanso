//! Synthetic pages for the tests: boxes placed the way a typesetter would,
//! and the text they must copy as.
//!
//! Geometry (points): characters are 6 wide and 8 tall, touching inside a
//! word; words are 3 apart; lines are 11 apart; paragraphs are 8 further
//! apart; columns start 260 apart.

use super::model::{PageText, TextChar};
use crate::model::Quad;

pub const CHAR_W: f64 = 6.0;
pub const CHAR_H: f64 = 8.0;
pub const WORD_GAP: f64 = 3.0;
pub const LINE_PITCH: f64 = 11.0;
pub const PARA_GAP: f64 = 8.0;
pub const COLUMN_PITCH: f64 = 260.0;
pub const MARGIN: f64 = 40.0;

pub type Words = Vec<String>;
pub type Para = Vec<Words>;
pub type Column = Vec<Para>;

/// A character box on the unrotated page.
pub struct Placed {
    pub ch: char,
    pub rect: [f64; 4],
    /// A space comes before this character, as PDFium reports it.
    pub space_before: bool,
}

/// Lays out columns of paragraphs of lines of words on an unrotated page and
/// returns the boxes with the page size.
pub fn place(columns: &[Column]) -> (Vec<Placed>, f64, f64) {
    let mut out = Vec::new();
    let mut bottom: f64 = 0.0;
    for (ci, col) in columns.iter().enumerate() {
        let x0 = MARGIN + ci as f64 * COLUMN_PITCH;
        let mut y = MARGIN;
        for (pi, para) in col.iter().enumerate() {
            if pi > 0 {
                y += PARA_GAP;
            }
            for line in para {
                let mut x = x0;
                for (wi, word) in line.iter().enumerate() {
                    if wi > 0 {
                        x += WORD_GAP;
                    }
                    for (ci, ch) in word.chars().enumerate() {
                        out.push(Placed {
                            ch,
                            rect: [x, y, x + CHAR_W, y + CHAR_H],
                            space_before: wi > 0 && ci == 0,
                        });
                        x += CHAR_W;
                    }
                }
                y += LINE_PITCH;
            }
        }
        bottom = bottom.max(y);
    }
    let w = MARGIN * 2.0 + columns.len().max(1) as f64 * COLUMN_PITCH;
    (out, w, bottom + MARGIN)
}

/// What copying all of `columns` gives.
pub fn expected(columns: &[Column]) -> String {
    columns
        .iter()
        .map(|c| {
            c.iter()
                .map(|p| p.iter().map(|l| l.join(" ")).collect::<Vec<_>>().join("\n"))
                .collect::<Vec<_>>()
                .join("\n\n")
        })
        .collect::<Vec<_>>()
        .join("\n\n")
}

/// Turns the page `rotation` degrees clockwise, the way a viewer displays a
/// page with `/Rotate`. Written out by hand from what the rotation does to a
/// page, not from the layout's own maths.
pub fn rotate_point(rotation: u16, w0: f64, h0: f64, u: f64, v: f64) -> (f64, f64) {
    match rotation % 360 {
        0 => (u, v),
        90 => (h0 - v, u),
        180 => (w0 - u, h0 - v),
        _ => (v, w0 - u),
    }
}

/// A displayed page from boxes on the unrotated page.
pub fn page_from(page: u32, placed: &[Placed], w0: f64, h0: f64, rotation: u16) -> PageText {
    let (w, h) = if rotation % 180 == 90 {
        (h0, w0)
    } else {
        (w0, h0)
    };
    let chars = placed
        .iter()
        .map(|p| {
            let (ax, ay) = rotate_point(rotation, w0, h0, p.rect[0], p.rect[1]);
            let (bx, by) = rotate_point(rotation, w0, h0, p.rect[2], p.rect[3]);
            TextChar {
                ch: p.ch,
                quad: Quad::new(
                    ax.min(bx) / w,
                    ay.min(by) / h,
                    ax.max(bx) / w,
                    ay.max(by) / h,
                ),
                turns: 0,
                space_before: p.space_before,
            }
        })
        .collect();
    PageText {
        page,
        width: w,
        height: h,
        rotation,
        chars,
    }
}

/// A page of the given columns, rotated.
pub fn page(page_no: u32, columns: &[Column], rotation: u16) -> PageText {
    let (placed, w, h) = place(columns);
    page_from(page_no, &placed, w, h, rotation)
}

/// One column of one paragraph from plain lines of text.
pub fn column_of(lines: &[&str]) -> Column {
    vec![
        lines
            .iter()
            .map(|l| l.split(' ').map(str::to_owned).collect())
            .collect(),
    ]
}

/// The centre of glyph `index`'s box on `page`, as a pointer position.
pub fn centre(doc: &super::TextDoc, page: u32, index: usize) -> super::Point {
    let q = doc.page(page).expect("page").glyphs[index].quad.0;
    super::Point {
        page,
        x: (q[0] + q[2]) / 2.0,
        y: (q[1] + q[3]) / 2.0,
    }
}
