//! Overview: Kanso's home page (issue #240). One page with two views:
//! **Today** ("what needs me now") and **This week** ("where did my week go,
//! and what's ahead"), assembled from the library when it is opened.
//!
//! It replaces the Morning Brief and the Weekly Review and is complete
//! without AI: the lede is written by [`lede`] from counts alone. The
//! pieces are pure where they can be ([`agenda`], [`density`], [`lede`],
//! [`summary`]) and read the store where they must ([`today`], [`week`]).
//!
//! Locked notes and notes in locked notebooks never appear: every store
//! query this calls leaves them out. `#ai-exclude` notes may show on the
//! page, but never reach a model.

pub mod agenda;
pub mod density;
pub mod lede;
mod meeting;
pub mod model;
mod past;
mod snapshot;
pub mod summary;
mod today;
mod week;

pub use meeting::{
    create_note as create_meeting_note, link_note as link_meeting_note, links as meeting_links,
};
pub use model::*;
pub use past::{dismiss_past, past_note};
pub use snapshot::{load_snapshot, save_snapshot, snapshot_key};
pub use today::{ActivityItem, Options, today};
pub use week::week;
