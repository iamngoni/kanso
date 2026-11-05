//! The reader's side of the ink attachment (issue #43, step 10): reading the
//! strokes in whenever the note is read again. The attachment itself, and
//! writing it, are `kanso_store::pdf_ink`, which the phone uses too.

use kanso_engine::lock::SealedInk;
use kanso_store::pdf_ink::{Held, INK_SEALED, read_ink_with};

use super::*;

impl PdfReader {
    /// Reads the ink attachment the block names, if it names one, and takes
    /// the strokes in. Run whenever the note is read again; it reads the
    /// attachment again only when its revision has moved.
    pub(super) fn load_ink_file(&mut self) {
        let Some(annots) = self.notes.annots.as_mut() else {
            return;
        };
        let Some(id) = annots.ink_file_id() else {
            self.ink_rev = None;
            return;
        };
        let rev_now = self.store.file_meta(&id).ok().flatten().and_then(|m| m.rev);
        if !annots.ink_pending && rev_now.is_some() && rev_now == self.ink_rev {
            return;
        }
        let files = SealedInk::reading(self.file_keys.book().clone());
        match read_ink_with(&self.store, &id, &files) {
            Held::Found(file, rev) => {
                if annots.absorb_ink(&file).is_ok() {
                    self.ink_rev = rev;
                }
            }
            Held::Missing => {}
            Held::Sealed => self.notes.notice = Some(INK_SEALED.to_owned()),
            Held::Unreadable(why) => {
                self.notes.notice = Some(format!("The pencil marks could not be read: {why}"));
            }
        }
        self.pencil_follow_note();
    }
}
