//! Open loops, the morning brief and the weekly review.

use kanso_mobile::{
    BriefPrefs, BriefView, KansoError, LoopAction, LoopProposal, OpenLoopsBoard, PlannedNotice,
    ReviewSaved, ReviewView,
};

use super::library::KansoLibrary;

impl KansoLibrary {
    pub fn open_loops(&self) -> Result<OpenLoopsBoard, KansoError> {
        self.0.open_loops()
    }

    pub fn close_loop(
        &self,
        note_id: String,
        key: String,
        line: u32,
        action: LoopAction,
    ) -> Result<bool, KansoError> {
        self.0.close_loop(&note_id, &key, line, action)
    }

    /// Keeps the proposals the person ticked; how many were added.
    pub fn approve_ai_loops(&self, picks: Vec<LoopProposal>) -> Result<u32, KansoError> {
        self.0.approve_ai_loops(picks)
    }

    pub fn brief(&self) -> Result<BriefView, KansoError> {
        self.0.brief()
    }

    pub fn review(&self, from: Option<String>) -> Result<ReviewView, KansoError> {
        self.0.review(from)
    }

    pub fn save_review(&self, from: String, replace: bool) -> Result<ReviewSaved, KansoError> {
        self.0.save_review(from, replace)
    }

    pub fn brief_prefs(&self) -> Result<BriefPrefs, KansoError> {
        self.0.brief_prefs()
    }

    pub fn set_brief_prefs(&self, prefs: BriefPrefs) -> Result<BriefPrefs, KansoError> {
        self.0.set_brief_prefs(prefs)
    }

    pub fn brief_plan(&self, can_notify: bool) -> Result<Vec<PlannedNotice>, KansoError> {
        self.0.brief_plan(can_notify)
    }
}
