//! An SVG drawing as a PNG, for "Copy as image".

use resvg::tiny_skia::{Pixmap, Transform};

/// The longest side of the picture, in pixels.
const MAX_SIDE: f32 = 8192.0;

/// `markup` drawn at `scale` times its size, as PNG bytes; the picture is
/// kept within [`MAX_SIDE`] pixels on a side. Text uses the fonts of
/// [`crate::set_fonts`]. Pure.
pub fn svg_to_png(markup: &str, scale: f32) -> Result<Vec<u8>, String> {
    let options = usvg::Options {
        fontdb: crate::svg::fonts(),
        ..usvg::Options::default()
    };
    let tree = usvg::Tree::from_str(markup, &options).map_err(|err| err.to_string())?;
    let size = tree.size();
    if !(size.width() > 0.0 && size.height() > 0.0 && scale.is_finite() && scale > 0.0) {
        return Err("The drawing has no size.".into());
    }
    let scale = scale.min(MAX_SIDE / size.width().max(size.height()));
    let width = (size.width() * scale).ceil().max(1.0) as u32;
    let height = (size.height() * scale).ceil().max(1.0) as u32;
    let mut pixmap = Pixmap::new(width, height).ok_or("The drawing is too large.")?;
    resvg::render(
        &tree,
        Transform::from_scale(scale, scale),
        &mut pixmap.as_mut(),
    );
    pixmap.encode_png().map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" viewBox="0 0 40 20" width="40" height="20"><rect width="40" height="20" fill="#336699"/></svg>"##;

    #[test]
    fn a_drawing_becomes_a_png_at_its_scale() {
        let png = svg_to_png(SVG, 2.0).unwrap();
        assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
        let size = |bytes: &[u8]| {
            (
                u32::from_be_bytes(bytes[16..20].try_into().unwrap()),
                u32::from_be_bytes(bytes[20..24].try_into().unwrap()),
            )
        };
        assert_eq!(size(&png), (80, 40));
        // Huge scales are kept in bounds, bad input is refused.
        assert!(size(&svg_to_png(SVG, 1.0e6).unwrap()).0 <= 8192);
        assert!(svg_to_png("not svg", 1.0).is_err());
        assert!(svg_to_png(SVG, 0.0).is_err());
    }
}
