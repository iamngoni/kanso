//! The reader's side of the ink attachment (issue #43, step 10): reading the
//! strokes in whenever the note is read again. The attachment itself, and
//! writing it, are `kanso_store::pdf_ink`, which the phone uses too.

use kanso_store::pdf_ink::{Held, read_ink_file};

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
        match read_ink_file(&self.store, &id) {
            Held::Found(file, rev) => {
                if annots.absorb_ink(&file).is_ok() {
                    self.ink_rev = rev;
                }
            }
            Held::Missing => {}
            Held::Unreadable(why) => {
                self.notes.notice = Some(format!("The pencil marks could not be read: {why}"));
            }
        }
        self.pencil_follow_note();
    }
}
