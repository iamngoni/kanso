//! People and projects (issue #200): ordinary notes marked `kanso: person`
//! or `kanso: project`, shown with live sections built from the rest of the
//! library, and suggested when notes keep mentioning a name.
//!
//! - [`page`]: a page as a [`crate::pages::PageSource`].
//! - [`detect`], [`suggest`], [`ai_detect`]: finding names worth a page.
//!   Detection suggests; nothing is created or merged without the person.
//! - [`actions`]: making a page, adding an alias, merging two pages.
//! - [`settings`]: Settings › People & Projects.

pub mod actions;
pub mod ai_detect;
pub mod detect;
pub mod page;
pub mod settings;
pub mod suggest;

pub use actions::{ActionError, Merged, add_alias, create_page, keep_book, merge};
pub use page::{EntityPage, PageHeader};
pub use settings::{DEFAULT_NOTEBOOK, EntitySettings};
pub use suggest::Suggestion;
