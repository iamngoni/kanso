//! Managing the library's notebooks and tags, as the desktop's sidebar
//! menus do: rename, delete, icon, color, new tag, tag color.

use kanso_core::model::{BOOK_COLOR_FIELD, BOOK_ICON_FIELD};
use kanso_core::{DocType, TagColor};
use kanso_engine::catalogue::{book_colors, book_icons};
use kanso_store::{Origin, Put};
use serde_json::{Value, json};

use crate::boundary::guard;
use crate::errors::KansoError;
use crate::library::Library;
use crate::model::{ColorChoice, IconChoice, TagRow};

impl Library {
    /// Renames notebook `id`. The name is trimmed; an empty one is refused
    /// ([`KansoError::Invalid`]). A locked notebook stays locked, and a
    /// notebook keeps its job (Inbox, Journal, Templates) however it is
    /// renamed. [`KansoError::NotFound`] when there is no such notebook.
    pub fn rename_notebook(&self, id: &str, name: String) -> Result<(), KansoError> {
        guard(|| {
            let name = name.trim();
            if name.is_empty() {
                return Err(KansoError::invalid("A notebook needs a name."));
            }
            self.notebook_exists(id)?;
            self.inner.store.put(
                DocType::Book,
                json!({"_id": id, "name": name}),
                Put::Merge,
                Origin::Ui,
            )?;
            Ok(())
        })
    }

    /// Deletes notebook `id`, as the desktop does: the notes in it and in the
    /// notebooks inside it move to the trash (they are not deleted), and the
    /// notebooks inside it are deleted with it. [`KansoError::NotFound`] when
    /// there is no such notebook.
    pub fn delete_notebook(&self, id: &str) -> Result<(), KansoError> {
        guard(|| {
            self.notebook_exists(id)?;
            self.inner.store.delete(id, Origin::Ui)?;
            Ok(())
        })
    }

    /// Sets notebook `id`'s icon, one of [`Library::notebook_icons`] by its
    /// name (or an icon theme's `<theme>/<icon>`; anything else is refused),
    /// or `None` for the default. A whole write, so `None`
    /// takes the field away; the lock and the job of the notebook stay.
    pub fn set_notebook_icon(&self, id: &str, icon: Option<String>) -> Result<(), KansoError> {
        guard(|| {
            let icon = match icon {
                Some(icon) => Some(valid_icon(&icon)?),
                None => None,
            };
            self.set_book_field(id, BOOK_ICON_FIELD, icon)
        })
    }

    /// Sets notebook `id`'s color, `#rrggbb` or `#rgb` in any case (kept as
    /// lowercase `#rrggbb`), or `None` for no color.
    pub fn set_notebook_color(&self, id: &str, color: Option<String>) -> Result<(), KansoError> {
        guard(|| {
            let color = match color {
                Some(color) => Some(valid_color(&color)?),
                None => None,
            };
            self.set_book_field(id, BOOK_COLOR_FIELD, color)
        })
    }

    /// Makes a tag named `name` (trimmed; an empty name is refused) and
    /// returns it as the sidebar lists it.
    pub fn create_tag(&self, name: String) -> Result<TagRow, KansoError> {
        guard(|| {
            let name = named(&name, "A tag needs a name.")?;
            let created = self.inner.store.put(
                DocType::Tag,
                json!({"name": name}),
                Put::Upsert,
                Origin::Ui,
            )?;
            self.tags()?
                .into_iter()
                .find(|tag| tag.id == created.id)
                .ok_or(KansoError::NotFound)
        })
    }

    /// Renames tag `id` (trimmed; empty is refused). [`KansoError::NotFound`]
    /// when there is no such tag.
    pub fn rename_tag(&self, id: &str, name: String) -> Result<(), KansoError> {
        guard(|| {
            let name = named(&name, "A tag needs a name.")?;
            self.tag_exists(id)?;
            self.inner.store.put(
                DocType::Tag,
                json!({"_id": id, "name": name}),
                Put::Merge,
                Origin::Ui,
            )?;
            Ok(())
        })
    }

    /// Deletes tag `id`: notes keep their content and lose the tag.
    /// [`KansoError::NotFound`] when there is no such tag.
    pub fn delete_tag(&self, id: &str) -> Result<(), KansoError> {
        guard(|| {
            self.tag_exists(id)?;
            self.inner.store.delete(id, Origin::Ui)?;
            Ok(())
        })
    }

    /// Sets tag `id`'s color to one of [`Library::tag_colors`] by its name,
    /// as the desktop writes it. Any other name is refused.
    pub fn set_tag_color(&self, id: &str, color: String) -> Result<(), KansoError> {
        guard(|| {
            let color: TagColor = serde_json::from_value(Value::String(color.clone()))
                .map_err(|_| KansoError::invalid(format!("“{color}” isn't a tag color.")))?;
            self.tag_exists(id)?;
            self.inner.store.put(
                DocType::Tag,
                json!({"_id": id, "color": tag_color_name(color)}),
                Put::Merge,
                Origin::Ui,
            )?;
            Ok(())
        })
    }

    /// The icons a notebook can show whose label, name or keywords start
    /// with each word of `query`, in the picker's order; an empty query is
    /// all of them.
    pub fn notebook_icons(query: String) -> Vec<IconChoice> {
        book_icons::matching(&query)
            .into_iter()
            .map(|choice| IconChoice {
                name: choice.name.to_owned(),
                label: choice.label.to_owned(),
            })
            .collect()
    }

    /// The swatches the notebook color picker offers (the tag colors but the
    /// default), in the desktop's order; `name` is the tag color's name.
    pub fn notebook_colors() -> Vec<ColorChoice> {
        book_colors::presets()
            .into_iter()
            .map(|color| ColorChoice {
                name: tag_color_name(color),
                rgb: book_colors::preset_rgb(color),
            })
            .collect()
    }

    /// `text` as a notebook color kept as `#rrggbb` in lowercase, if it is
    /// `#rrggbb` or `#rgb` (the `#` optional, any case, spaces around it
    /// ignored); what `set_notebook_color` would keep.
    pub fn parse_color(text: String) -> Option<String> {
        book_colors::parse_hex(&text).map(book_colors::hex)
    }

    /// The colors a tag can be, in the desktop's order, with the color each
    /// is drawn in.
    pub fn tag_colors() -> Vec<ColorChoice> {
        TagColor::ALL
            .into_iter()
            .map(|color| ColorChoice {
                name: tag_color_name(color),
                rgb: color.rgb(),
            })
            .collect()
    }

    fn notebook_exists(&self, id: &str) -> Result<(), KansoError> {
        if DocType::of_id(id) == Some(DocType::Book)
            && self.inner.store.books()?.iter().any(|book| book.id == id)
        {
            Ok(())
        } else {
            Err(KansoError::NotFound)
        }
    }

    fn tag_exists(&self, id: &str) -> Result<(), KansoError> {
        if DocType::of_id(id) == Some(DocType::Tag)
            && self.inner.store.tags()?.iter().any(|tag| tag.id == id)
        {
            Ok(())
        } else {
            Err(KansoError::NotFound)
        }
    }

    /// Sets notebook `id`'s `field` to `value`, or takes it away. A whole
    /// write, since a merge can't remove a field (the desktop's
    /// `set_book_field`).
    fn set_book_field(
        &self,
        id: &str,
        field: &str,
        value: Option<String>,
    ) -> Result<(), KansoError> {
        self.notebook_exists(id)?;
        let store = &self.inner.store;
        let doc = store.get(id)?.ok_or(KansoError::NotFound)?;
        let mut book = doc.to_api(Default::default());
        let fields = book.as_object_mut().ok_or(KansoError::NotFound)?;
        match value {
            Some(value) => fields.insert(field.into(), value.into()),
            None => fields.remove(field),
        };
        store.put(DocType::Book, book, Put::Upsert, Origin::Ui)?;
        Ok(())
    }
}

/// `name` trimmed, or [`KansoError::Invalid`] saying `empty` when nothing is
/// left.
fn named<'a>(name: &'a str, empty: &str) -> Result<&'a str, KansoError> {
    let name = name.trim();
    if name.is_empty() {
        Err(KansoError::invalid(empty))
    } else {
        Ok(name)
    }
}

/// An icon name the notebook can keep: one of the catalogue's, or an icon
/// theme's `<theme>/<icon>`.
fn valid_icon(icon: &str) -> Result<String, KansoError> {
    book_icons::chosen_name(Some(icon.trim()))
        .ok_or_else(|| KansoError::invalid(format!("“{}” isn't a notebook icon.", icon.trim())))
}

/// A notebook color as it is kept: `#rrggbb` in lowercase, from `#rrggbb` or
/// `#rgb` in any case.
fn valid_color(color: &str) -> Result<String, KansoError> {
    book_colors::parse_hex(color)
        .map(book_colors::hex)
        .ok_or_else(|| KansoError::invalid("A color is #rrggbb or #rgb."))
}

/// The JSON name of a tag color, as the desktop writes it.
fn tag_color_name(color: TagColor) -> String {
    serde_json::to_value(color)
        .ok()
        .and_then(|v| v.as_str().map(str::to_owned))
        .unwrap_or_else(|| "default".into())
}
