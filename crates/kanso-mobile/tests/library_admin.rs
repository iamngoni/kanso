//! Managing notebooks and tags: rename, delete, icon, color, new tag and tag
//! color. Locked notebooks are in `tests/actions_locked.rs`.

mod common;

use common::*;
use kanso_mobile::{KansoError, Library, NewNote, NoteChange, NotebookRow};
use proptest::prelude::*;

fn row(library: &Library, id: &str) -> NotebookRow {
    library
        .notebooks()
        .unwrap()
        .into_iter()
        .find(|row| row.id == id)
        .expect("the notebook is listed")
}

fn is_invalid<T>(result: &Result<T, KansoError>) -> bool {
    matches!(result, Err(KansoError::Invalid { .. }))
}

fn invalid<T: std::fmt::Debug>(result: Result<T, KansoError>) {
    assert!(
        matches!(result, Err(KansoError::Invalid { .. })),
        "{result:?}"
    );
}

// no-prop-test: the error paths, one by one; renaming over any name is a
// property below.
#[test]
fn renaming_a_notebook() {
    let phone = lone_phone();
    let library = &phone.library;
    let id = library.create_notebook("Work".into(), None).unwrap().id;
    library.rename_notebook(&id, "  Projects ".into()).unwrap();
    assert_eq!(row(library, &id).name, "Projects");
    invalid(library.rename_notebook(&id, "   ".into()));
    invalid(library.rename_notebook(&id, String::new()));
    assert_eq!(
        row(library, &id).name,
        "Projects",
        "a refused name changes nothing"
    );
    assert_eq!(
        library.rename_notebook("book:nope", "X".into()),
        Err(KansoError::NotFound)
    );
    assert_eq!(
        library.rename_notebook("trash", "X".into()),
        Err(KansoError::NotFound)
    );
    let note = note(library, "A note", "text");
    assert_eq!(
        library.rename_notebook(&note, "X".into()),
        Err(KansoError::NotFound)
    );
}

// no-prop-test: a fixed tree of notebooks and the exact rows.
#[test]
fn a_row_says_whether_notebooks_are_inside_it() {
    let phone = lone_phone();
    let library = &phone.library;
    let parent = library.create_notebook("Parent".into(), None).unwrap().id;
    let child = library
        .create_notebook("Child".into(), Some(parent.clone()))
        .unwrap()
        .id;
    let leaf = library
        .create_notebook("Leaf".into(), Some(child.clone()))
        .unwrap()
        .id;
    let other = library.create_notebook("Zed".into(), None).unwrap().id;
    let flags: Vec<(String, bool)> = library
        .notebooks()
        .unwrap()
        .into_iter()
        .map(|row| (row.id, row.has_children))
        .collect();
    assert_eq!(
        flags,
        [(parent, true), (child, true), (leaf, false), (other, false)]
    );
}

// no-prop-test: the store's cascade, on a fixed tree.
#[test]
fn deleting_a_notebook_trashes_its_notes_and_removes_what_is_inside() {
    let phone = lone_phone();
    let library = &phone.library;
    let outer = library.create_notebook("Outer".into(), None).unwrap().id;
    let inner = library
        .create_notebook("Inner".into(), Some(outer.clone()))
        .unwrap()
        .id;
    let keep = library.create_notebook("Keep".into(), None).unwrap().id;
    let make = |book: &str, title: &str| {
        library
            .create(NewNote {
                notebook_id: Some(book.to_owned()),
                title: title.to_owned(),
                body: "x".into(),
                ..NewNote::default()
            })
            .unwrap()
            .id
    };
    let (a, b, c) = (make(&outer, "a"), make(&inner, "b"), make(&keep, "c"));
    library.delete_notebook(&outer).unwrap();
    let names: Vec<String> = library
        .notebooks()
        .unwrap()
        .into_iter()
        .map(|r| r.name)
        .collect();
    assert_eq!(names, ["Keep"]);
    assert_eq!(library.note(&a).unwrap().notebook_id, "trash");
    assert_eq!(library.note(&b).unwrap().notebook_id, "trash");
    assert_eq!(library.note(&c).unwrap().notebook_id, keep);
    assert_eq!(library.delete_notebook(&outer), Err(KansoError::NotFound));
    assert_eq!(library.delete_notebook("trash"), Err(KansoError::NotFound));
}

// no-prop-test: set, replace, clear and refuse, in order; normalising any
// color is a property below.
#[test]
fn a_notebook_takes_and_loses_an_icon_and_a_color() {
    let phone = lone_phone();
    let library = &phone.library;
    let id = library.create_notebook("Work".into(), None).unwrap().id;
    assert_eq!(
        (row(library, &id).icon, row(library, &id).color),
        (None, None)
    );
    library
        .set_notebook_icon(&id, Some("briefcase".into()))
        .unwrap();
    library
        .set_notebook_color(&id, Some("#2185D0".into()))
        .unwrap();
    let shown = row(library, &id);
    assert_eq!(shown.icon.as_deref(), Some("briefcase"));
    assert_eq!(shown.color.as_deref(), Some("#2185d0"));
    assert_eq!(shown.name, "Work");
    // One field changes without the other.
    library.set_notebook_icon(&id, None).unwrap();
    let shown = row(library, &id);
    assert_eq!(
        (shown.icon, shown.color.as_deref()),
        (None, Some("#2185d0"))
    );
    library
        .set_notebook_color(&id, Some("#fa0".into()))
        .unwrap();
    assert_eq!(row(library, &id).color.as_deref(), Some("#ffaa00"));
    library.set_notebook_color(&id, None).unwrap();
    assert_eq!(row(library, &id).color, None);
    for bad in [
        "", "   ", "red", "#12", "#12345", "#1234567", "#ggg", "##fff",
    ] {
        invalid(library.set_notebook_color(&id, Some(bad.into())));
    }
    invalid(library.set_notebook_icon(&id, Some(String::new())));
    invalid(library.set_notebook_icon(&id, Some(" ".into())));
    invalid(library.set_notebook_icon(&id, Some("no-such-icon".into())));
    invalid(library.set_notebook_icon(&id, Some("Briefcase".into())));
    invalid(library.set_notebook_icon(&id, Some("x".repeat(65))));
    // An icon theme's icon is kept as `<theme>/<icon>`.
    library
        .set_notebook_icon(&id, Some("mytheme/star".into()))
        .unwrap();
    assert_eq!(row(library, &id).icon.as_deref(), Some("mytheme/star"));
    assert_eq!(
        row(library, &id).color,
        None,
        "refused values changed nothing"
    );
    assert_eq!(
        library.set_notebook_icon("book:nope", None),
        Err(KansoError::NotFound)
    );
    assert_eq!(
        library.set_notebook_color("trash", None),
        Err(KansoError::NotFound)
    );
}

// no-prop-test: the catalogue's constants and a search over them.
#[test]
fn the_catalogues_list_icons_colors_and_parse_a_color() {
    let all = Library::notebook_icons(String::new());
    assert!(all.len() > 20);
    assert_eq!(all[0].name, "notebook");
    let work = Library::notebook_icons("off".into());
    assert!(work.iter().any(|icon| icon.name == "briefcase"), "{work:?}");
    assert!(work.len() < all.len());
    assert!(Library::notebook_icons("zzzzzz".into()).is_empty());
    // Every icon offered can be set.
    let phone = lone_phone();
    let id = phone.library.create_notebook("A".into(), None).unwrap().id;
    for icon in &all {
        phone
            .library
            .set_notebook_icon(&id, Some(icon.name.clone()))
            .unwrap();
    }
    let colors = Library::notebook_colors();
    assert_eq!(colors.len(), 13, "the tag colors but the default");
    assert_eq!((colors[0].name.as_str(), colors[0].rgb), ("red", 0xdb2828));
    for color in &colors {
        let hex = Library::parse_color(format!("#{:06X}", color.rgb)).unwrap();
        phone
            .library
            .set_notebook_color(&id, Some(hex.clone()))
            .unwrap();
        assert_eq!(row(&phone.library, &id).color, Some(hex));
    }
    assert_eq!(
        Library::parse_color(" #FA0 ".into()).as_deref(),
        Some("#ffaa00")
    );
    assert_eq!(Library::parse_color("nope".into()), None);
    assert_eq!(Library::parse_color(String::new()), None);
}

// no-prop-test: create, rename, recolor, delete, with their error paths.
#[test]
fn managing_tags() {
    let phone = lone_phone();
    let library = &phone.library;
    let made = library.create_tag("  work/acme ".into()).unwrap();
    assert_eq!((made.name.as_str(), made.count), ("work/acme", 0));
    assert_eq!(made.color, 0x8a8f98, "a new tag has the default color");
    invalid(library.create_tag("  ".into()));
    library.rename_tag(&made.id, " clients ".into()).unwrap();
    assert_eq!(library.tags().unwrap()[0].name, "clients");
    invalid(library.rename_tag(&made.id, String::new()));
    assert_eq!(
        library.rename_tag("tag:nope", "x".into()),
        Err(KansoError::NotFound)
    );

    library.set_tag_color(&made.id, "red".into()).unwrap();
    assert_eq!(library.tags().unwrap()[0].color, 0xdb2828);
    for choice in Library::tag_colors() {
        library
            .set_tag_color(&made.id, choice.name.clone())
            .unwrap();
        assert_eq!(library.tags().unwrap()[0].color, choice.rgb);
    }
    invalid(library.set_tag_color(&made.id, "chartreuse".into()));
    invalid(library.set_tag_color(&made.id, "#ff0000".into()));
    assert_eq!(
        library.set_tag_color("tag:nope", "red".into()),
        Err(KansoError::NotFound)
    );

    // A note keeps its content and loses the tag.
    let id = note(library, "Tagged", "text");
    library
        .change(
            vec![id.clone()],
            NoteChange::AddTag {
                tag_id: made.id.clone(),
            },
        )
        .unwrap();
    assert_eq!(library.note(&id).unwrap().tags, vec![made.id.clone()]);
    library.delete_tag(&made.id).unwrap();
    assert!(library.tags().unwrap().is_empty());
    let view = library.note(&id).unwrap();
    assert_eq!((view.body.as_deref(), view.tags.len()), (Some("text"), 0));
    assert_eq!(library.delete_tag(&made.id), Err(KansoError::NotFound));
    assert_eq!(library.delete_tag(&id), Err(KansoError::NotFound));
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// Round trip: a name set is the name read back, trimmed; a name with
    /// nothing in it is refused and changes nothing.
    #[test]
    fn prop_a_rename_round_trips(name in "[ \\t]{0,3}[a-zA-Z0-9 ]{0,20}[ ]{0,3}") {
        let phone = lone_phone();
        let library = &phone.library;
        let book = library.create_notebook("Start".into(), None).unwrap().id;
        let tag = library.create_tag("start".into()).unwrap().id;
        let trimmed = name.trim();
        let books = library.rename_notebook(&book, name.clone());
        let tags = library.rename_tag(&tag, name.clone());
        if trimmed.is_empty() {
            prop_assert!(is_invalid(&books));
            prop_assert!(is_invalid(&tags));
            prop_assert_eq!(row(library, &book).name, "Start");
            prop_assert_eq!(&library.tags().unwrap()[0].name, "start");
        } else {
            prop_assert_eq!(books, Ok(()));
            prop_assert_eq!(tags, Ok(()));
            prop_assert_eq!(row(library, &book).name, trimmed);
            prop_assert_eq!(&library.tags().unwrap()[0].name, trimmed);
        }
    }

    /// Any `#rgb` or `#rrggbb`, in any case, is kept as lowercase `#rrggbb`
    /// (each digit of `#rgb` doubled); anything else is refused and the
    /// color stays.
    #[test]
    fn prop_a_color_is_normalised(
        short in "#[0-9a-fA-F]{3}",
        long in "#[0-9a-fA-F]{6}",
        bad in "#([0-9a-fA-F]{1,2}|[0-9a-fA-F]{4,5}|[0-9a-fA-F]{7,9})",
    ) {
        let phone = lone_phone();
        let library = &phone.library;
        let id = library.create_notebook("Work".into(), None).unwrap().id;
        library.set_notebook_color(&id, Some(long.clone())).unwrap();
        prop_assert_eq!(row(library, &id).color, Some(long.to_ascii_lowercase()));
        library.set_notebook_color(&id, Some(short.clone())).unwrap();
        let doubled: String = short[1..].chars().flat_map(|c| [c, c]).collect();
        prop_assert_eq!(row(library, &id).color, Some(format!("#{}", doubled.to_ascii_lowercase())));
        let kept = row(library, &id).color;
        prop_assert!(is_invalid(&library.set_notebook_color(&id, Some(bad))));
        prop_assert_eq!(row(library, &id).color, kept);
    }
}
