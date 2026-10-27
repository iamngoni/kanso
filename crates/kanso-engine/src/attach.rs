//! Files that become attachments: the images and PDFs pasted, dropped or
//! shared into a note. What a file is is decided here, from its extension
//! and its first bytes, and what the note needs to show it is the Markdown
//! this module returns. Pure apart from the store write, and from
//! [`to_png`], which decodes with the `image` crate.

use std::borrow::Cow;

use base64::Engine as _;
use kanso_core::DocType;
use kanso_core::links::LINK_SCHEME;
use kanso_core::model::MAX_ATTACHMENT_SIZE;
use kanso_store::{Origin, Put, Store, StoreError};

/// What a pasted or dropped image is stored as: the API's image types as
/// they are, others converted to PNG.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Stored {
    /// Kept as it is, with this content type and file extension.
    AsIs(&'static str, &'static str),
    /// Decoded and stored as a PNG.
    Png,
}

/// How to store an image, from its file extension or format name
/// (`png`, `jpg`, `tiff`…); `None` when it isn't an image Kanso reads.
pub fn stored_as(extension: &str) -> Option<Stored> {
    Some(match extension.to_ascii_lowercase().as_str() {
        "png" => Stored::AsIs("image/png", "png"),
        "jpg" | "jpeg" => Stored::AsIs("image/jpeg", "jpg"),
        "gif" => Stored::AsIs("image/gif", "gif"),
        "svg" => Stored::AsIs("image/svg+xml", "svg"),
        "heic" => Stored::AsIs("image/heic", "heic"),
        "heif" => Stored::AsIs("image/heif", "heif"),
        "webp" | "bmp" | "tif" | "tiff" | "ico" => Stored::Png,
        _ => return None,
    })
}

/// The content type of a PDF attachment.
const PDF: &str = "application/pdf";

/// For a file called an image whose bytes aren't one.
const NOT_AN_IMAGE: &str = "That file isn't an image Kanso can read.";

/// What a file is, ready to be written: the bytes to store (an image
/// already converted), their content type and the name the attachment gets.
struct Prepared {
    name: String,
    content_type: &'static str,
    data: Vec<u8>,
    /// What the user is told it is, "image" or "PDF".
    kind: &'static str,
}

/// An attached file: its ID and the Markdown that shows it in a note.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Attached {
    /// The attachment's ID (`file:…`).
    pub id: String,
    /// `![](inkdrop://file:…)` for an image, `[name.pdf](inkdrop://file:…)`
    /// for a PDF.
    pub markdown: String,
}

/// Stores an image (its bytes, and its extension or format name) as an
/// attachment, converting what the API doesn't take to PNG. Returns the
/// attachment's ID (`file:…`), or what to tell the user.
pub fn attach_image(
    store: &Store,
    bytes: &[u8],
    extension: &str,
    name: Option<&str>,
) -> Result<String, String> {
    attach_image_as(store, bytes, extension, name, None)
}

/// [`attach_image`] under the given ID (`file:…`), or a fresh one. A file
/// already stored under that ID is the one an earlier try wrote, and is
/// kept as it is.
pub fn attach_image_as(
    store: &Store,
    bytes: &[u8],
    extension: &str,
    name: Option<&str>,
    id: Option<&str>,
) -> Result<String, String> {
    let prepared = prepare_image(bytes, extension, name)?;
    write(store, &prepared, id)
        .map_err(|err| format!("Couldn't attach the {}: {err}", prepared.kind))
}

/// Stores a PDF as an attachment, named `name` (its extension made `.pdf`)
/// or "document.pdf", under the given ID (`file:…`) or a fresh one. The
/// bytes must be a PDF. Returns the attachment's ID, or what to tell the
/// user.
pub fn attach_pdf(
    store: &Store,
    bytes: &[u8],
    name: &str,
    id: Option<&str>,
) -> Result<String, String> {
    let prepared = prepare_pdf(bytes, Some(name))?;
    write(store, &prepared, id)
        .map_err(|err| format!("Couldn't attach the {}: {err}", prepared.kind))
}

/// Stores a shared file as an attachment if it is an image or a PDF. The
/// file's extension (from `name`, else from `type_hint`, the content type
/// the sharing app gave) and its first bytes decide which; the hint alone
/// never does, and a file whose bytes are a PDF is a PDF whatever it is
/// called. Anything else is refused with a sentence for the user.
pub fn attach_file(
    store: &Store,
    bytes: &[u8],
    name: &str,
    type_hint: Option<&str>,
    id: Option<&str>,
) -> Result<Attached, String> {
    let prepared = prepare_file(bytes, name, type_hint)?;
    stored(store, &prepared, id)
        .map_err(|err| format!("Couldn't attach the {}: {err}", prepared.kind))
}

/// Writes `prepared` and says how a note shows it.
fn stored(store: &Store, prepared: &Prepared, id: Option<&str>) -> Result<Attached, StoreError> {
    let id = write(store, prepared, id)?;
    let markdown = if prepared.content_type == PDF {
        file_markdown(&id, &prepared.name)
    } else {
        image_markdown(&id)
    };
    Ok(Attached { id, markdown })
}

/// [`attach_file`] split in two, so a caller can tell a file refused for
/// what it is from a write that failed: the first half's error is a
/// sentence for the user, the second's is the store's.
pub(crate) fn prepare_and_store(
    store: &Store,
    bytes: &[u8],
    name: &str,
    type_hint: Option<&str>,
    id: &str,
) -> Result<Result<Attached, StoreError>, String> {
    let prepared = prepare_file(bytes, name, type_hint)?;
    Ok(stored(store, &prepared, Some(id)))
}

/// What `bytes` named `name` is, when it is an image or a PDF.
fn prepare_file(bytes: &[u8], name: &str, type_hint: Option<&str>) -> Result<Prepared, String> {
    let extension = std::path::Path::new(name)
        .extension()
        .and_then(|ext| ext.to_str())
        .map(str::to_ascii_lowercase)
        .or_else(|| type_hint.and_then(extension_of_type))
        .unwrap_or_default();
    let sniffed = sniff(bytes);
    if sniffed == Some("pdf") {
        return prepare_pdf(bytes, Some(name));
    }
    if let Some(format) = sniffed {
        return prepare_image(bytes, format, Some(name));
    }
    if extension == "pdf" {
        return Err("That file isn't a PDF Kanso can read.".into());
    }
    match stored_as(&extension) {
        // No signature: an SVG is text, and the rest of the formats read
        // by their own decoder.
        Some(Stored::AsIs(_, "svg")) if !looks_like_svg(bytes) => {
            return Err(NOT_AN_IMAGE.into());
        }
        // A PNG, JPEG, GIF or HEIC that isn't one of them would show broken.
        Some(Stored::AsIs(_, ext)) if ext != "svg" => return Err(NOT_AN_IMAGE.into()),
        Some(_) => return prepare_image(bytes, &extension, Some(name)),
        None => {}
    }
    Err(
        "Only images and PDFs can be attached: PNG, JPEG, GIF, SVG, HEIC, WebP, TIFF, BMP or PDF."
            .into(),
    )
}

fn prepare_image(bytes: &[u8], extension: &str, name: Option<&str>) -> Result<Prepared, String> {
    let Some(kind) = stored_as(extension) else {
        return Err(
            "Only images can be attached: PNG, JPEG, GIF, SVG, HEIC, WebP, TIFF or BMP.".into(),
        );
    };
    let (data, content_type, ext) = match kind {
        Stored::AsIs(content_type, ext) => (Cow::Borrowed(bytes), content_type, ext),
        Stored::Png => (Cow::Owned(to_png(bytes)?), "image/png", "png"),
    };
    if data.len() > MAX_ATTACHMENT_SIZE {
        return Err(too_large(data.len(), MAX_ATTACHMENT_SIZE));
    }
    Ok(Prepared {
        name: format!("{}.{ext}", stem_of(name, "image")),
        content_type,
        data: data.into_owned(),
        kind: "image",
    })
}

fn prepare_pdf(bytes: &[u8], name: Option<&str>) -> Result<Prepared, String> {
    if !bytes.starts_with(b"%PDF-") {
        return Err("That file isn't a PDF Kanso can read.".into());
    }
    if bytes.len() > MAX_ATTACHMENT_SIZE {
        return Err(too_large_file("PDF", bytes.len(), MAX_ATTACHMENT_SIZE));
    }
    Ok(Prepared {
        name: format!("{}.pdf", stem_of(name, "document")),
        content_type: PDF,
        data: bytes.to_vec(),
        kind: "PDF",
    })
}

/// Writes the attachment: under `id` when one is given and nothing is
/// there yet, else under a fresh one.
fn write(store: &Store, prepared: &Prepared, id: Option<&str>) -> Result<String, StoreError> {
    if let Some(id) = id
        && let Some(existing) = store.get(id)?
        && existing.doc_type == DocType::File
    {
        return Ok(existing.id);
    }
    let encoded = base64::engine::general_purpose::STANDARD.encode(&prepared.data);
    let mut doc = serde_json::json!({
        "name": prepared.name,
        "contentType": prepared.content_type,
        "publicIn": [],
        "_attachments": {"index": {"content_type": prepared.content_type, "data": encoded}},
    });
    if let Some(id) = id {
        doc["_id"] = id.into();
    }
    store
        .put(DocType::File, doc, Put::Upsert, Origin::Ui)
        .map(|written| written.id)
}

/// The name without its extension: at most 100 characters, with control
/// characters made spaces; `default` when `name` has none to give.
fn stem_of(name: Option<&str>, default: &str) -> String {
    name.and_then(|n| {
        std::path::Path::new(n)
            .file_stem()?
            .to_str()
            .map(str::to_owned)
    })
    .map(|stem| {
        stem.chars()
            .map(|c| if c.is_control() { ' ' } else { c })
            .collect::<String>()
    })
    .filter(|s| !s.trim().is_empty())
    .unwrap_or_else(|| default.into())
    .chars()
    .take(100)
    .collect()
}

/// A file name without its extension, as an attachment would be named
/// (empty when there is none).
pub(crate) fn file_stem(name: &str) -> String {
    stem_of(Some(name), "")
}

/// The extension a content type stands for, for the types Kanso reads.
fn extension_of_type(content_type: &str) -> Option<String> {
    let essence = content_type.split(';').next()?.trim().to_ascii_lowercase();
    if essence == PDF {
        return Some("pdf".into());
    }
    Some(match essence.strip_prefix("image/")? {
        "svg+xml" => "svg".to_owned(),
        "x-icon" | "vnd.microsoft.icon" => "ico".to_owned(),
        subtype => subtype.to_owned(),
    })
}

/// Whether `bytes` start like an SVG: markup, after any spaces and a byte
/// order mark.
fn looks_like_svg(bytes: &[u8]) -> bool {
    let head = &bytes[..bytes.len().min(512)];
    let head = head.strip_prefix(b"\xef\xbb\xbf").unwrap_or(head);
    head.trim_ascii_start().starts_with(b"<")
}

/// The format `bytes` start like, by the extension that names it: only the
/// formats with a signature (`svg`, a text format, has none).
fn sniff(bytes: &[u8]) -> Option<&'static str> {
    const SIGNATURES: [(&[u8], &str); 7] = [
        (b"%PDF-", "pdf"),
        (b"\x89PNG\r\n\x1a\n", "png"),
        (b"\xff\xd8\xff", "jpg"),
        (b"GIF8", "gif"),
        (b"II*\0", "tiff"),
        (b"MM\0*", "tiff"),
        (b"BM", "bmp"),
    ];
    if let Some((_, format)) = SIGNATURES.iter().find(|(sig, _)| bytes.starts_with(sig)) {
        // "BM" alone is two letters of text as often as a bitmap.
        if *format != "bmp" || bytes.len() >= 26 {
            return Some(format);
        }
    }
    if bytes.len() >= 12 && &bytes[..4] == b"RIFF" && &bytes[8..12] == b"WEBP" {
        return Some("webp");
    }
    if bytes.len() >= 12 && &bytes[4..8] == b"ftyp" {
        return match &bytes[8..12] {
            b"heic" | b"heix" | b"hevc" | b"hevx" => Some("heic"),
            b"heif" | b"mif1" | b"msf1" => Some("heif"),
            _ => None,
        };
    }
    None
}

/// An image in another format, as a PNG.
fn to_png(bytes: &[u8]) -> Result<Vec<u8>, String> {
    let image =
        image::load_from_memory(bytes).map_err(|err| format!("Couldn't read the image: {err}"))?;
    let mut out = std::io::Cursor::new(Vec::new());
    image
        .write_to(&mut out, image::ImageFormat::Png)
        .map_err(|err| format!("Couldn't convert the image: {err}"))?;
    Ok(out.into_inner())
}

/// The Markdown that shows attachment `file_id` (`file:…`).
pub fn image_markdown(file_id: &str) -> String {
    format!("![]({LINK_SCHEME}{file_id})")
}

/// The Markdown that links attachment `file_id` (`file:…`), shown as `name`:
/// `[name](inkdrop://file:…)`. Backslashes and square brackets in the name
/// are escaped and a line break becomes a space, so the name stays the
/// link's one line of text.
pub fn file_markdown(file_id: &str, name: &str) -> String {
    format!("[{}]({LINK_SCHEME}{file_id})", link_text(name))
}

/// `text` as the text of a Markdown link: trimmed, with backslashes and
/// square brackets escaped and each line break a space.
pub(crate) fn link_text(text: &str) -> String {
    let mut label = String::with_capacity(text.len());
    for c in text.trim().chars() {
        match c {
            '\\' | '[' | ']' => {
                label.push('\\');
                label.push(c);
            }
            '\n' | '\r' => label.push(' '),
            _ => label.push(c),
        }
    }
    label
}

/// "4.2 MB": a size for a message.
pub fn megabytes(bytes: usize) -> String {
    let mb = bytes as f64 / (1024. * 1024.);
    if mb >= 10. {
        format!("{mb:.0} MB")
    } else {
        format!("{mb:.1} MB")
    }
}

/// The notice for an image over the attachment limit.
pub fn too_large(bytes: usize, limit: usize) -> String {
    too_large_file("image", bytes, limit)
}

/// The notice for a file of `kind` ("image", "PDF") over the attachment
/// limit.
pub fn too_large_file(kind: &str, bytes: usize, limit: usize) -> String {
    format!(
        "That {kind} is {}. Attachments can be up to {}.",
        megabytes(bytes),
        megabytes(limit)
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        (dir, store)
    }

    /// An image of `w` by `h` pixels encoded as `format`.
    fn encoded(w: u32, h: u32, seed: u8, format: image::ImageFormat) -> Vec<u8> {
        let image = image::RgbaImage::from_fn(w, h, |x, y| {
            image::Rgba([seed.wrapping_add(x as u8), y as u8, seed, 255])
        });
        let mut out = std::io::Cursor::new(Vec::new());
        image::DynamicImage::ImageRgba8(image)
            .write_to(&mut out, format)
            .unwrap();
        out.into_inner()
    }

    const PDF_BYTES: &[u8] = b"%PDF-1.7\n1 0 obj\n<<>>\nendobj\n%%EOF\n";

    #[test]
    fn image_types() {
        assert_eq!(stored_as("PNG"), Some(Stored::AsIs("image/png", "png")));
        assert_eq!(stored_as("jpeg"), Some(Stored::AsIs("image/jpeg", "jpg")));
        assert_eq!(stored_as("tiff"), Some(Stored::Png));
        assert_eq!(stored_as("pdf"), None);
        assert_eq!(image_markdown("file:abc"), "![](inkdrop://file:abc)");
        assert_eq!(
            too_large(12 * 1024 * 1024, 10 * 1024 * 1024),
            "That image is 12 MB. Attachments can be up to 10 MB."
        );
        assert_eq!(
            too_large_file("PDF", 12 * 1024 * 1024, 10 * 1024 * 1024),
            "That PDF is 12 MB. Attachments can be up to 10 MB."
        );
        assert_eq!(megabytes(4_404_019), "4.2 MB");
    }

    #[test]
    fn images_become_attachments() {
        let (_dir, store) = store();
        let png = encoded(3, 2, 7, image::ImageFormat::Png);
        let id = attach_image(&store, &png, "png", Some("Screen Shot.png")).unwrap();
        let (content_type, blob) = store.attachment(&id).unwrap().unwrap();
        assert_eq!(content_type, "image/png");
        assert_eq!(blob.data, png);
        let doc = store.get(&id).unwrap().unwrap();
        assert_eq!(doc.fields["name"], "Screen Shot.png");

        // TIFF isn't one of the API's types: stored as a PNG of the same picture.
        let tiff = encoded(4, 5, 9, image::ImageFormat::Tiff);
        let id = attach_image(&store, &tiff, "tiff", None).unwrap();
        let (content_type, blob) = store.attachment(&id).unwrap().unwrap();
        assert_eq!(content_type, "image/png");
        let back = image::load_from_memory(&blob.data).unwrap().to_rgba8();
        let original = image::load_from_memory(&tiff).unwrap().to_rgba8();
        assert_eq!(back, original);
        assert_eq!(store.get(&id).unwrap().unwrap().fields["name"], "image.png");

        assert!(
            attach_image(&store, b"%PDF", "pdf", None)
                .unwrap_err()
                .contains("Only images")
        );
        let big = vec![0u8; 12 * 1024 * 1024 + 1];
        assert_eq!(
            attach_image(&store, &big, "png", None).unwrap_err(),
            "That image is 12 MB. Attachments can be up to 10 MB."
        );
    }

    #[test]
    fn a_given_id_is_used_and_kept_on_a_second_try() {
        let (_dir, store) = store();
        let png = encoded(2, 2, 1, image::ImageFormat::Png);
        let id = attach_image_as(&store, &png, "png", None, Some("file:cap-a-0")).unwrap();
        assert_eq!(id, "file:cap-a-0");
        let again = attach_image_as(&store, &png, "png", None, Some("file:cap-a-0")).unwrap();
        assert_eq!(again, id);
        // An ID that isn't a file's is refused by the store, in words.
        let err = attach_image_as(&store, &png, "png", None, Some("note:x")).unwrap_err();
        assert!(err.starts_with("Couldn't attach the image: "), "{err}");
    }

    #[test]
    fn pdfs_become_attachments() {
        let (_dir, store) = store();
        let id = attach_pdf(&store, PDF_BYTES, "Quarterly report.PDF", None).unwrap();
        let (content_type, blob) = store.attachment(&id).unwrap().unwrap();
        assert_eq!(content_type, "application/pdf");
        assert_eq!(blob.data, PDF_BYTES);
        assert_eq!(
            store.get(&id).unwrap().unwrap().fields["name"],
            "Quarterly report.pdf"
        );
        let id = attach_pdf(&store, PDF_BYTES, "", Some("file:cap-b-1")).unwrap();
        assert_eq!(id, "file:cap-b-1");
        assert_eq!(
            store.get(&id).unwrap().unwrap().fields["name"],
            "document.pdf"
        );

        assert_eq!(
            attach_pdf(&store, b"<html>", "a.pdf", None).unwrap_err(),
            "That file isn't a PDF Kanso can read."
        );
        let mut big = b"%PDF-".to_vec();
        big.resize(11 * 1024 * 1024, 0);
        assert_eq!(
            attach_pdf(&store, &big, "a.pdf", None).unwrap_err(),
            "That PDF is 11 MB. Attachments can be up to 10 MB."
        );
        let long = format!("{}.pdf", "n".repeat(300));
        let id = attach_pdf(&store, PDF_BYTES, &long, None).unwrap();
        let name = store.get(&id).unwrap().unwrap().fields["name"]
            .as_str()
            .unwrap()
            .to_owned();
        assert_eq!(name.chars().count(), 104);
    }

    #[test]
    fn shared_files_are_told_apart_by_extension_and_bytes() {
        let (_dir, store) = store();
        let png = encoded(2, 3, 5, image::ImageFormat::Png);
        let tiff = encoded(2, 3, 5, image::ImageFormat::Tiff);

        let pdf =
            attach_file(&store, PDF_BYTES, "Plan.pdf", Some("application/pdf"), None).unwrap();
        assert_eq!(pdf.markdown, format!("[Plan.pdf](inkdrop://{})", pdf.id));
        // A PDF whose name or hint says otherwise is still a PDF.
        let odd = attach_file(&store, PDF_BYTES, "scan", Some("image/png"), None).unwrap();
        assert_eq!(store.attachment(&odd.id).unwrap().unwrap().0, PDF);
        assert_eq!(
            store.get(&odd.id).unwrap().unwrap().fields["name"],
            "scan.pdf"
        );

        let image = attach_file(&store, &png, "a.png", Some("image/png"), None).unwrap();
        assert_eq!(image.markdown, format!("![](inkdrop://{})", image.id));
        // Bytes decide over the extension: a TIFF called .png is converted.
        let tiff = attach_file(&store, &tiff, "b.png", None, None).unwrap();
        assert_eq!(store.attachment(&tiff.id).unwrap().unwrap().0, "image/png");
        // No extension: the hint names an SVG, which has no signature.
        let svg = attach_file(&store, b"<svg/>", "logo", Some("image/svg+xml"), None).unwrap();
        assert_eq!(
            store.attachment(&svg.id).unwrap().unwrap().0,
            "image/svg+xml"
        );

        let refused = |bytes: &[u8], name: &str, hint: Option<&str>| {
            attach_file(&store, bytes, name, hint, None).unwrap_err()
        };
        assert_eq!(
            refused(b"not a pdf", "a.pdf", Some("application/pdf")),
            "That file isn't a PDF Kanso can read."
        );
        assert!(refused(b"PK\x03\x04", "a.zip", None).starts_with("Only images and PDFs"));
        // Called an image, and not one.
        assert_eq!(
            refused(b"hello", "a.png", Some("image/png")),
            "That file isn't an image Kanso can read."
        );
        assert_eq!(
            refused(b"hello", "a.svg", None),
            "That file isn't an image Kanso can read."
        );
        assert!(
            refused(b"hello", "notes.txt", Some("text/plain")).starts_with("Only images and PDFs")
        );
        // A hint of PDF with no bytes behind it is no PDF.
        assert!(refused(b"hello", "x", Some("application/pdf")).contains("isn't a PDF"));
    }

    #[test]
    fn file_links_escape_the_name() {
        assert_eq!(
            file_markdown("file:a", "Q1 report.pdf"),
            "[Q1 report.pdf](inkdrop://file:a)"
        );
        assert_eq!(
            file_markdown("file:a", "[draft] a\\b\nc.pdf"),
            "[\\[draft\\] a\\\\b c.pdf](inkdrop://file:a)"
        );
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(32))]

        /// Round trip: an image in a format Kanso converts comes back as
        /// the same pixels.
        #[test]
        fn prop_converted_images_keep_their_pixels(w in 1u32..12, h in 1u32..12, seed: u8, format in prop::sample::select(vec![
            (image::ImageFormat::Tiff, "tiff"), (image::ImageFormat::Bmp, "bmp"),
        ])) {
            let (_dir, store) = store();
            let bytes = encoded(w, h, seed, format.0);
            let id = attach_image(&store, &bytes, format.1, None).unwrap();
            let (_, blob) = store.attachment(&id).unwrap().unwrap();
            let back = image::load_from_memory(&blob.data).unwrap().to_rgba8();
            prop_assert_eq!(back, image::load_from_memory(&bytes).unwrap().to_rgba8());
        }
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        /// Invariant: whatever the bytes, the name and the hint, a file
        /// stored as `application/pdf` starts with `%PDF-`, and nothing
        /// that starts with it is stored as anything else.
        #[test]
        fn prop_only_pdfs_are_stored_as_pdfs(
            body in prop::collection::vec(any::<u8>(), 0..64),
            pdf_head in any::<bool>(),
            name in "[a-zA-Z0-9 ._\\[\\]-]{0,24}",
            hint in prop::option::of(prop::sample::select(vec![
                "application/pdf", "image/png", "image/jpeg", "text/plain", "application/octet-stream",
            ])),
        ) {
            let (_dir, store) = store();
            let mut bytes = Vec::new();
            if pdf_head {
                bytes.extend_from_slice(b"%PDF-");
            }
            bytes.extend_from_slice(&body);
            if let Ok(attached) = attach_file(&store, &bytes, &name, hint, None) {
                let (content_type, _) = store.attachment(&attached.id).unwrap().unwrap();
                prop_assert_eq!(content_type == PDF, bytes.starts_with(b"%PDF-"));
                let stored = store.get(&attached.id).unwrap().unwrap();
                prop_assert_eq!(stored.fields["name"].as_str().unwrap().ends_with(".pdf"), content_type == PDF);
                prop_assert!(stored.fields["name"].as_str().unwrap().chars().count() <= 128);
            } else if pdf_head {
                // A real PDF is refused only for its size, which this isn't.
                prop_assert!(false, "a PDF was refused");
            }
        }

        /// Invariant: the link text never ends its brackets early, and the
        /// link ends with the file's address.
        #[test]
        fn prop_file_links_keep_one_line_of_text(name in "[a-z\\[\\]\\\\\n\r ]{0,24}") {
            let link = file_markdown("file:x", &name);
            prop_assert!(link.starts_with('['));
            prop_assert!(link.ends_with("](inkdrop://file:x)"));
            prop_assert!(!link.contains('\n') && !link.contains('\r'));
            let label = &link[1..link.len() - "](inkdrop://file:x)".len()];
            let mut chars = label.chars();
            while let Some(c) = chars.next() {
                match c {
                    '\\' => prop_assert!(matches!(chars.next(), Some('\\' | '[' | ']'))),
                    '[' | ']' => prop_assert!(false, "unescaped bracket in {label:?}"),
                    _ => {}
                }
            }
        }
    }
}
