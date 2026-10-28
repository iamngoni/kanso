//! Links and getting around: the outline, backlinks, unlinked mentions,
//! Open Note and the command palette. The decisions are the facade's.

use flutter_rust_bridge::frb;
use kanso_mobile::{
    KansoError, LinkRef, Linking, OutlineHeading, PaletteContext, PaletteSection, Saved,
};

#[frb(opaque)]
pub struct KansoLinking(Linking);

impl KansoLinking {
    pub(super) fn new(linking: Linking) -> KansoLinking {
        KansoLinking(linking)
    }

    /// The headings of `text`, in order.
    #[frb(sync)]
    pub fn outline(&self, text: String) -> Vec<OutlineHeading> {
        self.0.outline(&text)
    }

    /// The index of the heading the caret on `line` is under, if any.
    #[frb(sync)]
    pub fn current_heading(&self, headings: Vec<OutlineHeading>, line: u32) -> Option<u32> {
        self.0.current_heading(&headings, line)
    }

    pub fn backlinks(&self, id: String) -> Result<Vec<LinkRef>, KansoError> {
        self.0.backlinks(&id)
    }

    pub fn unlinked_mentions(&self, id: String) -> Result<Vec<LinkRef>, KansoError> {
        self.0.unlinked_mentions(&id)
    }

    pub fn link_mention(
        &self,
        target_id: String,
        mention_id: String,
        line: u32,
    ) -> Result<Option<Saved>, KansoError> {
        self.0.link_mention(&target_id, &mention_id, line)
    }

    pub fn quick_open(
        &self,
        query: String,
        open: Option<String>,
    ) -> Result<Vec<PaletteSection>, KansoError> {
        self.0.quick_open(&query, open)
    }

    pub fn palette(
        &self,
        query: String,
        context: PaletteContext,
    ) -> Result<Vec<PaletteSection>, KansoError> {
        self.0.palette(&query, context)
    }
}
