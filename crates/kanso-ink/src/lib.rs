//! Cross-platform ink core.
//!
//! Owns everything about a sketch that must be identical on every platform: the
//! canonical (CBOR) document format, stroke geometry (smoothing + pressure→width),
//! and headless rasterization for previews/export. Native layers only capture raw
//! stylus input and present GPU frames; they delegate the math and the format here.

pub mod format;
pub mod geometry;
#[cfg(feature = "gpu")]
pub mod gpu;
pub mod raster;
pub mod tessellate;

pub use format::*;

#[cfg(test)]
mod tests {
    use super::format::*;
    use super::raster;

    fn sample_doc() -> SketchDoc {
        let mut doc = SketchDoc::new();
        doc.background = Background::Dotted;
        doc.elements.push(Element::Stroke(Stroke {
            points: vec![
                Point {
                    x: 5.0,
                    y: 5.0,
                    pressure: 0.4,
                    tilt: 0.0,
                    t: 0.0,
                },
                Point {
                    x: 60.0,
                    y: 40.0,
                    pressure: 0.9,
                    tilt: 0.0,
                    t: 1.0,
                },
                Point {
                    x: 95.0,
                    y: 12.0,
                    pressure: 0.7,
                    tilt: 0.0,
                    t: 2.0,
                },
            ],
            color: Rgba {
                r: 20,
                g: 20,
                b: 20,
                a: 255,
            },
            base_width: 3.0,
            tool: Tool::Pen,
        }));
        doc
    }

    #[test]
    fn cbor_roundtrips() {
        let doc = sample_doc();
        let bytes = doc.to_cbor();
        let back = SketchDoc::from_cbor(&bytes).expect("decode");
        assert_eq!(back.format_version, FORMAT_VERSION);
        assert_eq!(back.background, Background::Dotted);
        assert_eq!(back.elements.len(), 1);
    }

    #[test]
    fn renders_png_preview() {
        let doc = sample_doc();
        let png = raster::render_preview(&doc, 128, 96).expect("png");
        // PNG magic number.
        assert!(png.len() > 8 && &png[1..4] == b"PNG");
    }
}
