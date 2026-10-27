//! What the sidebar shows, decided by the engine's rules the desktop's
//! sidebar also follows: which notebooks show and which are open, and the
//! tag chips and the nested tag tree. The phone keeps [`FoldState`] and the
//! open tag paths itself (they are plain data) and passes them in.

use std::collections::HashSet;

use kanso_core::tags::{TagInfo, is_within, tag_tree};
use kanso_engine::list::{book_path, notebook_tree};
use kanso_engine::sidebar::{
    BookFolds, TAG_LIMIT, flat_tag_ids, visible_book_rows, visible_tag_rows, visible_tags,
};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::{FoldState, TagPlaces, TagRow, TagTreeRow, VisibleNotebook};
use crate::notes::notebook_rows;

impl Library {
    /// The notebook rows that show, top to bottom, with whether each is open.
    /// `folds` is what the phone kept; `showing` is the notebook whose notes
    /// are listed, which keeps the notebooks around it open unless they were
    /// folded with their arrows while it showed.
    pub fn visible_notebooks(
        &self,
        folds: FoldState,
        showing: Option<String>,
    ) -> Result<Vec<VisibleNotebook>, KansoError> {
        guard(|| {
            let books = self.inner.store.books()?;
            let tree = notebook_tree(&books);
            let depths: Vec<usize> = tree.iter().map(|(depth, _)| *depth).collect();
            let rows = notebook_rows(&books);
            let folds = to_folds(&folds);
            let around = around(&books, showing.as_deref());
            let open = |ix: usize| folds.is_open(&tree[ix].1.id, showing.as_deref(), &around);
            Ok(visible_book_rows(&depths, &open)
                .into_iter()
                .map(|(ix, parent)| VisibleNotebook {
                    row: rows[ix].clone(),
                    open: parent && open(ix),
                    parent,
                })
                .collect())
        })
    }

    /// The folds after a tap on notebook `id`'s arrow: it folds if it shows
    /// open, else it opens.
    pub fn toggle_fold(
        &self,
        folds: FoldState,
        id: String,
        showing: Option<String>,
    ) -> Result<FoldState, KansoError> {
        guard(|| {
            let books = self.inner.store.books()?;
            let around = around(&books, showing.as_deref());
            let mut folds = to_folds(&folds);
            folds.toggle(&id, showing.as_deref(), &around);
            Ok(from_folds(&folds))
        })
    }

    /// The sidebar's tags: chips for the tags that are neither nested nor
    /// have tags nested in them (all of them while `all` is set or there are
    /// few, else the most used few, and `selected` (a tag's ID) on top if it
    /// isn't among them), and the tree of nested tags with its open levels.
    /// `open_paths` are the levels opened with their arrows (any case);
    /// `showing` is the tag path or tag name whose notes are listed, which
    /// keeps the levels around it open.
    pub fn tag_places(
        &self,
        all: bool,
        selected: Option<String>,
        open_paths: Vec<String>,
        showing: Option<String>,
    ) -> Result<TagPlaces, KansoError> {
        guard(|| {
            let store = &self.inner.store;
            let tags = store.tags()?;
            let infos: Vec<TagInfo> = tags
                .iter()
                .map(|tag| TagInfo {
                    id: tag.id.clone(),
                    name: tag.name.clone(),
                })
                .collect();
            let tree = tag_tree(&infos, &store.tag_links()?);
            let flat = flat_tag_ids(&tree);
            let flat_tags: Vec<_> = tags
                .iter()
                .filter(|tag| flat.contains(tag.id.as_str()))
                .collect();
            let selected = selected
                .as_deref()
                .and_then(|id| flat_tags.iter().position(|tag| tag.id == id));
            let counts: Vec<u32> = flat_tags.iter().map(|tag| tag.count).collect();
            let chips = visible_tags(&counts, TAG_LIMIT, selected, all)
                .into_iter()
                .map(|ix| {
                    let tag = flat_tags[ix];
                    TagRow {
                        id: tag.id.clone(),
                        name: tag.name.clone(),
                        color: tag.color.rgb(),
                        count: tag.count,
                    }
                })
                .collect();
            let nested: Vec<_> = tree
                .iter()
                .filter(|node| !node.children.is_empty())
                .cloned()
                .collect();
            let opened: HashSet<String> =
                open_paths.iter().map(|path| path.to_lowercase()).collect();
            let open = |path: &str| {
                opened.contains(&path.to_lowercase())
                    || showing.as_deref().is_some_and(|shown| {
                        is_within(shown, path) && !shown.eq_ignore_ascii_case(path)
                    })
            };
            let rows = visible_tag_rows(&nested, &open)
                .into_iter()
                .map(|(depth, node)| {
                    let parent = !node.children.is_empty();
                    TagTreeRow {
                        path: node.path.clone(),
                        label: node.label.clone(),
                        tag_id: node.tag_id.clone(),
                        count: node.count,
                        depth: depth as u32,
                        parent,
                        open: parent && open(&node.path),
                    }
                })
                .collect();
            Ok(TagPlaces {
                chips,
                more: counts.len() > TAG_LIMIT,
                tree: rows,
            })
        })
    }
}

/// The notebooks `showing` is inside.
fn around(books: &[kanso_core::Book], showing: Option<&str>) -> HashSet<String> {
    showing
        .map(|id| {
            book_path(id, books)
                .iter()
                .filter(|book| book.id != id)
                .map(|book| book.id.clone())
                .collect()
        })
        .unwrap_or_default()
}

fn to_folds(state: &FoldState) -> BookFolds {
    BookFolds::with_state(
        state.open.iter().cloned().collect(),
        state
            .folded_at
            .clone()
            .map(|at| (at, state.folded.iter().cloned().collect())),
    )
}

fn from_folds(folds: &BookFolds) -> FoldState {
    let mut open: Vec<String> = folds.open.iter().cloned().collect();
    open.sort();
    let (folded_at, mut folded) = match folds.folded_around() {
        Some((at, folded)) => (Some(at.to_owned()), folded.iter().cloned().collect()),
        None => (None, Vec::new()),
    };
    folded.sort();
    FoldState {
        open,
        folded_at,
        folded,
    }
}
