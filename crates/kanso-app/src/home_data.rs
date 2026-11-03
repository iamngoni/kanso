//! Notebook homepages on the desktop (issue #201), the side that is not
//! drawing: reading a homepage with the person's settings, rewriting its AI
//! sections with the model, and the decisions behind the settings. The page,
//! its sections, the map and the AI writing are the engine's
//! (`kanso_engine::pages::notebook`), which the phones use too.
//!
//! A locked notebook has no homepage on disk: closed it gives nothing, open
//! it is built in memory from the notes this session opened and dropped
//! with them. A locked note, a note in a locked notebook, a trashed note
//! and, for AI, a note tagged `#ai-exclude` are never a source.

use std::sync::Arc;

use kanso_core::now_ms;
use kanso_engine::pages::locked_home::locked_home;
use kanso_engine::pages::notebook::{HomeOptions, NotebookPage, open_home};
use kanso_engine::pages::notebook_model::{HomeView, MapView, OpenedNote};
use kanso_engine::pages::{self, RefreshPolicy, write::WriteContext};
use kanso_store::Store;

use crate::ai_feature::Maker;
use crate::config::AppConfig;

/// A homepage as the view shows it.
#[derive(Clone, Debug)]
pub struct Loaded {
    pub view: HomeView,
    /// Sections the person's refresh setting wants written now.
    pub due: Vec<String>,
    /// The notebook is locked and open this session: computed sections
    /// built in memory.
    pub locked_open: bool,
}

/// Whether the notebook AI sections are written at all.
pub fn ai_on(config: &AppConfig) -> bool {
    config.people_projects.notebook_ai(config.ai.enabled)
}

/// When AI sections are rewritten without being asked: the same setting as
/// people's pages.
pub fn policy(config: &AppConfig) -> RefreshPolicy {
    crate::people_data::policy(config)
}

/// Opens notebook `id`'s homepage. `show_all` names computed sections to
/// show in full. `opened` is the notes of a locked notebook this session has
/// open (none for a notebook that is not locked); for a locked notebook
/// without it, nothing is built.
pub fn load(
    store: &Store,
    id: &str,
    config: &AppConfig,
    show_all: &[&str],
    opened: Option<Vec<OpenedNote>>,
) -> kanso_store::Result<Option<Loaded>> {
    let options = HomeOptions {
        ai_on: ai_on(config),
        now: now_ms(),
    };
    if let Some(notes) = opened {
        let name = store
            .get(id)?
            .and_then(|d| {
                d.fields
                    .get("name")
                    .and_then(|n| n.as_str().map(str::to_owned))
            })
            .unwrap_or_default();
        let view = locked_home(
            &name,
            id,
            &notes,
            options.now,
            show_all.contains(&"open_tasks"),
        );
        return Ok(Some(Loaded {
            view,
            due: Vec::new(),
            locked_open: true,
        }));
    }
    let Some(page) = NotebookPage::load(store, id, options.now)? else {
        return Ok(None);
    };
    let view = open_home(store, &page, options, show_all)?;
    let due = pages::due_for_refresh(
        &view.page,
        policy(config),
        pages::OpenOptions {
            ai_on: options.ai_on,
            now: options.now,
        },
    );
    Ok(Some(Loaded {
        view,
        due,
        locked_open: false,
    }))
}

/// The notebook's map, laid out. Heavy: call off the main thread.
pub fn map(store: &Store, id: &str) -> kanso_store::Result<Option<MapView>> {
    let Some(page) = NotebookPage::load(store, id, now_ms())? else {
        return Ok(None);
    };
    Ok(Some(page.map(store)?))
}

/// Rewrites section `key` of notebook `id` (or every section that is not
/// kept, when `key` is `None`) with the model `maker` makes. The error is
/// the model's own words, for `page_chrome::ai_trouble` to put plainly.
pub fn refresh(
    store: &Arc<Store>,
    id: &str,
    key: Option<&str>,
    model: &str,
    maker: &Maker,
) -> Result<(), String> {
    let page = NotebookPage::load(store, id, now_ms())
        .map_err(|err| err.to_string())?
        .ok_or_else(|| "That notebook has no homepage.".to_owned())?;
    let shared = maker()?;
    let provider = shared
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let context = WriteContext::new(provider.as_ref(), model, now_ms());
    match key {
        Some(key) => pages::refresh_section(store, &page, key, &context).map(|_| ()),
        None => pages::refresh_all(store, &page, &context).map(|_| ()),
    }
    .map_err(|err| err.to_string())
}
