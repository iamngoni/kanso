//! Text in images: the words in image attachments, read on this device so
//! search finds a note by what its pictures say.
//!
//! The reading itself belongs to the platform: Apple's Vision framework on
//! a Mac, Vision again on an iPhone, ML Kit's text recognizer on Android.
//! This is the part they share. [`next_unread`] says which image to read
//! next and gives its bytes, the platform reads them wherever it can, and
//! [`keep`] stores the words in the file's `kansoOcr`
//! (`kanso-store/src/ocr.rs`), which syncs sealed with the file, so other
//! devices search it too without reading the image again. A phone reads
//! through a platform channel between the two calls; the desktop's
//! [`read_next`] does both with a [`TextReader`].
//!
//! Only images a note shows that may be read are offered: not those only a
//! locked note or a note tagged `#ai-exclude` shows (the store decides).

use kanso_store::Store;

/// Reads the words in an image.
pub trait TextReader: Send + Sync {
    /// Names the reader in the file's `kansoOcr`.
    fn engine(&self) -> &'static str;
    /// The image's words, line by line; empty when it has none.
    fn read(&self, image: &[u8]) -> Result<String, String>;
}

/// Where reading stands.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub enum Status {
    #[default]
    Off,
    /// Reading: how many images are left.
    Reading { left: usize },
    /// Every image is read.
    Idle,
}

/// An image waiting to be read: its file, the MD5 of the bytes (kept with
/// the words, so new bytes are read again) and the bytes.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ImageToRead {
    pub id: String,
    pub md5: String,
    pub bytes: Vec<u8>,
    /// How many images are waiting, this one included.
    pub waiting: usize,
}

/// How many images are asked of the store at once.
const LOOK_AHEAD: usize = 64;

/// The next image to read, newest first, with its bytes; `None` when none
/// is waiting. An image whose bytes are gone is kept as having no words, so
/// it is passed over.
pub fn next_unread(store: &Store) -> kanso_store::Result<Option<ImageToRead>> {
    let waiting = store.unread_images(LOOK_AHEAD)?;
    let count = waiting.len();
    for (ix, image) in waiting.into_iter().enumerate() {
        match store.attachment(&image.id)? {
            Some((_, attachment)) => {
                return Ok(Some(ImageToRead {
                    id: image.id,
                    md5: image.md5,
                    bytes: attachment.data,
                    waiting: count - ix,
                }));
            }
            None => {
                store.set_image_text(&image.id, &image.md5, "", "none")?;
            }
        }
    }
    Ok(None)
}

/// Keeps `text` as what image `id` says, read from the bytes with `md5` by
/// `engine`. Nothing is kept when the file is gone or its bytes changed
/// since: it is read again. Whether it was kept. An image that couldn't be
/// read is kept with no words, so it isn't tried again until its bytes
/// change.
pub fn keep(
    store: &Store,
    id: &str,
    md5: &str,
    text: &str,
    engine: &str,
) -> kanso_store::Result<bool> {
    store.set_image_text(id, md5, text.trim(), engine)
}

/// How many images are waiting to be read.
pub fn waiting(store: &Store) -> kanso_store::Result<usize> {
    Ok(store.unread_images(usize::MAX)?.len())
}

/// Reads the next unread image with `reader`, if there is one: returns how
/// many were waiting (this one included), 0 when none.
pub fn read_next(store: &Store, reader: &dyn TextReader) -> kanso_store::Result<usize> {
    let Some(image) = next_unread(store)? else {
        return Ok(0);
    };
    let text = reader.read(&image.bytes).unwrap_or_else(|err| {
        eprintln!("kanso: couldn't read the text in {}: {err}", image.id);
        String::new()
    });
    keep(store, &image.id, &image.md5, &text, reader.engine())?;
    Ok(image.waiting)
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::Engine as _;
    use kanso_core::DocType;
    use kanso_store::{Origin, Put};
    use proptest::prelude::*;
    use serde_json::json;
    use std::sync::Mutex;

    /// Says what it was given, and counts.
    struct Fake(Mutex<usize>, bool);

    impl TextReader for Fake {
        fn engine(&self) -> &'static str {
            "fake"
        }
        fn read(&self, image: &[u8]) -> Result<String, String> {
            *self.0.lock().unwrap() += 1;
            if self.1 {
                return Err("unreadable".into());
            }
            Ok(format!("words of {}", String::from_utf8_lossy(image)))
        }
    }

    fn store() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("k.db")).unwrap();
        (dir, store)
    }

    /// A file of `bytes`, and a note in a notebook that shows it.
    fn attach(store: &Store, bytes: &[u8], content_type: &str) -> String {
        let id = store
            .put(
                DocType::File,
                json!({
                    "name": "a",
                    "contentType": content_type,
                    "_attachments": {"index": {
                        "content_type": content_type,
                        "data": base64::engine::general_purpose::STANDARD.encode(bytes),
                    }},
                }),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap()
            .id;
        show(store, &id);
        id
    }

    fn show(store: &Store, file: &str) {
        let book = store
            .put(DocType::Book, json!({"name": "B"}), Put::Upsert, Origin::Ui)
            .unwrap()
            .id;
        store
            .put(
                DocType::Note,
                json!({"title": "Shows", "body": format!("![](inkdrop://{file})"), "bookId": book}),
                Put::Upsert,
                Origin::Ui,
            )
            .unwrap();
    }

    #[test]
    fn images_are_read_once_and_again_when_they_change() {
        let (_dir, store) = store();
        let png = attach(&store, b"one", "image/png");
        attach(&store, b"<svg/>", "image/svg+xml");
        let reader = Fake(Mutex::new(0), false);
        assert_eq!(read_next(&store, &reader).unwrap(), 1);
        assert_eq!(read_next(&store, &reader).unwrap(), 0);
        assert_eq!(*reader.0.lock().unwrap(), 1);
        assert_eq!(
            store.image_text(&png).unwrap().as_deref(),
            Some("words of one")
        );
        // New bytes: read again.
        let rev = store.get(&png).unwrap().unwrap().rev;
        store
            .put(
                DocType::File,
                json!({"_id": png, "_rev": rev, "_attachments": {"index": {
                    "content_type": "image/png",
                    "data": base64::engine::general_purpose::STANDARD.encode(b"two"),
                }}}),
                Put::Merge,
                Origin::Ui,
            )
            .unwrap();
        assert_eq!(read_next(&store, &reader).unwrap(), 1);
        assert_eq!(
            store.image_text(&png).unwrap().as_deref(),
            Some("words of two")
        );
    }

    #[test]
    fn an_unreadable_image_is_not_tried_forever() {
        let (_dir, store) = store();
        let png = attach(&store, b"x", "image/jpeg");
        let reader = Fake(Mutex::new(0), true);
        assert_eq!(read_next(&store, &reader).unwrap(), 1);
        assert_eq!(read_next(&store, &reader).unwrap(), 0);
        assert_eq!(store.image_text(&png).unwrap().as_deref(), Some(""));
    }

    #[test]
    fn a_phone_reads_between_the_two_calls() {
        let (_dir, store) = store();
        let png = attach(&store, b"photo", "image/png");
        assert_eq!(waiting(&store).unwrap(), 1);
        let image = next_unread(&store).unwrap().unwrap();
        assert_eq!(
            (image.id.as_str(), image.bytes.as_slice()),
            (png.as_str(), &b"photo"[..])
        );
        // Words read from other bytes are refused; the right ones are kept,
        // trimmed.
        assert!(!keep(&store, &png, "not-the-md5", "x", "mlkit").unwrap());
        assert!(keep(&store, &png, &image.md5, "  Quarterly plan \n", "mlkit").unwrap());
        assert_eq!(
            store.image_text(&png).unwrap().as_deref(),
            Some("Quarterly plan")
        );
        assert!(next_unread(&store).unwrap().is_none());
        assert_eq!(waiting(&store).unwrap(), 0);
    }

    proptest! {
        #![proptest_config(ProptestConfig { cases: 24, ..ProptestConfig::default() })]

        /// However many images there are, reading them one by one reads each
        /// once, newest first, counting down what is left.
        #[test]
        fn prop_every_image_is_read_exactly_once(count in 0usize..6) {
            let (_dir, store) = store();
            for n in 0..count {
                attach(&store, format!("img{n}").as_bytes(), "image/png");
            }
            let reader = Fake(Mutex::new(0), false);
            let mut left = count;
            while left > 0 {
                prop_assert_eq!(read_next(&store, &reader).unwrap(), left);
                left -= 1;
            }
            prop_assert_eq!(read_next(&store, &reader).unwrap(), 0);
            prop_assert_eq!(*reader.0.lock().unwrap(), count);
            prop_assert_eq!(waiting(&store).unwrap(), 0);
        }
    }
}
