//! The app's side of every host call: checked against the stored grant,
//! answered from the store through [`Scope`], and copied back as JSON.
//!
//! A call outside the grant is refused with `denied`, and counted: three in
//! a session and the supervisor turns the plugin off. A note the plugin may
//! not read is `not_found`, never `denied`, so a locked or `#ai-exclude`
//! note can't be told from a missing one. Changes to notes are staged in
//! the command's [`Batch`], never written here; a renderer may call
//! nothing at all. The clipboard history is a copy the app takes before
//! the request, and text for the clipboard is put there by the app after.

use std::collections::BTreeSet;
use std::path::Path;

use base64::Engine as _;
use kanso_core::model::{MAX_BODY_LEN, MAX_TITLE_LEN, NoteStatus};
use kanso_plugin_host::protocol::{CODE_CONFLICT, CODE_INVALID, CODE_LIMIT, Failure};
use kanso_store::{PluginCreate, PluginEdit, Store, plugin_data};
use serde_json::{Map, Value, json};

use crate::batch::{Batch, MAX_PLUGIN_DATA};
use crate::model::{ClipboardPermission, EditorPermission, Installed};
use crate::panel_model::Node;
use crate::scope::{ListQuery, MAX_ATTACHMENT_BYTES, Scope};

/// The largest settings a plugin may keep.
pub const MAX_SETTINGS_BYTES: usize = 64 * 1024;
/// The longest text `replaceSelection` may put in.
pub const MAX_REPLACEMENT_BYTES: usize = 1024 * 1024;
/// The longest text `clipboard.write` may put on the clipboard.
pub const MAX_CLIPBOARD_BYTES: usize = 1024 * 1024;

/// One copy in the clipboard history, as the app kept it: already
/// screened, newest first.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ClipEntry {
    pub text: String,
    /// Milliseconds since the epoch.
    pub copied_ms: i64,
}

/// Tokens a person saved for plugins, kept in the Keychain by the app.
pub trait Secrets {
    /// Token `name` saved for plugin `plugin_id`.
    fn get(&self, plugin_id: &str, name: &str) -> Option<String>;
}

/// The network proxy, as the app runs it ([`crate::net`]).
pub trait Network {
    /// `kanso.fetch` for `plugin_id`, to `hosts` only.
    fn fetch(
        &mut self,
        plugin_id: &str,
        hosts: &BTreeSet<String>,
        args: &Value,
    ) -> Result<Value, Failure>;
}

/// No network: what a renderer and tests get.
pub struct NoNetwork;

impl Network for NoNetwork {
    fn fetch(&mut self, _: &str, _: &BTreeSet<String>, _: &Value) -> Result<Value, Failure> {
        Err(Failure::denied("to use the network"))
    }
}

/// The selection in the open note, in bytes of its text.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Selection {
    pub text: String,
    pub start: usize,
    pub end: usize,
}

/// What the editor shows while a request runs.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Editor {
    pub note_id: Option<String>,
    pub selection: Option<Selection>,
}

/// A replacement for the selection, applied by the app as one edit.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SelectionEdit {
    pub note_id: String,
    pub start: usize,
    pub end: usize,
    pub text: String,
}

/// Which request is running.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Request {
    Command,
    Render,
    Press,
}

/// What a request left for the app to do once it returns.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outcome {
    pub batch: Batch,
    pub selection_edit: Option<SelectionEdit>,
    /// A new panel tree, already checked.
    pub panel: Option<Node>,
    /// Text to put on the clipboard.
    pub clipboard_write: Option<String>,
    /// Calls refused for want of a permission.
    pub refused: u32,
}

/// Answers one request's host calls for one plugin.
pub struct Router<'a> {
    pub store: &'a Store,
    pub plugin: &'a Installed,
    /// Where the plugin's settings live (`plugins/<id>`).
    pub plugin_dir: &'a Path,
    pub editor: &'a Editor,
    pub request: Request,
    pub secrets: &'a dyn Secrets,
    pub network: &'a mut dyn Network,
    /// The clipboard history, when the plugin may read it; empty otherwise.
    pub clipboard: &'a [ClipEntry],
    pub out: Outcome,
}

fn invalid(message: impl Into<String>) -> Failure {
    Failure::new(CODE_INVALID, message)
}

fn text_arg<'v>(args: &'v Value, key: &str) -> Result<&'v str, Failure> {
    args.get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| invalid(format!("{key} must be text")))
}

/// The fields a plugin may change, read from `notes.update`'s `fields`.
/// Anything else is refused. Pure.
pub fn edit_fields(fields: &Value) -> Result<PluginEdit, Failure> {
    let Some(fields) = fields.as_object() else {
        return Err(invalid("fields must be an object"));
    };
    let mut edit = PluginEdit::default();
    for (key, value) in fields {
        match key.as_str() {
            "title" => {
                let title = value
                    .as_str()
                    .ok_or_else(|| invalid("title must be text"))?;
                if title.chars().count() > MAX_TITLE_LEN {
                    return Err(invalid("the title is too long"));
                }
                edit.title = Some(title.to_owned());
            }
            "body" => {
                let body = value.as_str().ok_or_else(|| invalid("body must be text"))?;
                if body.chars().count() > MAX_BODY_LEN {
                    return Err(invalid("the text is too long"));
                }
                edit.body = Some(body.to_owned());
            }
            "tags" => {
                let tags = value
                    .as_array()
                    .and_then(|tags| {
                        tags.iter()
                            .map(|t| t.as_str().map(str::to_owned))
                            .collect::<Option<Vec<_>>>()
                    })
                    .ok_or_else(|| invalid("tags must be a list of tag IDs"))?;
                if tags.len() > 64 || tags.iter().any(|t| !t.starts_with("tag:")) {
                    return Err(invalid("tags must be a list of tag IDs"));
                }
                edit.tags = Some(tags);
            }
            "status" => {
                let status = value
                    .as_str()
                    .and_then(NoteStatus::parse)
                    .ok_or_else(|| invalid("status isn't one Kanso knows"))?;
                edit.status = Some(status.as_str().to_owned());
            }
            other => {
                return Err(invalid(format!(
                    "a plugin can change only title, body, tags and status, not {other}"
                )));
            }
        }
    }
    Ok(edit)
}

impl Router<'_> {
    fn grant(&self) -> &crate::model::Grant {
        &self.plugin.grant
    }

    /// Refuses and counts.
    fn refuse(&mut self, what: &str) -> Failure {
        self.out.refused += 1;
        Failure::denied(what)
    }

    fn scope(&self) -> Result<Scope<'_>, Failure> {
        Scope::new(self.store, self.grant(), self.editor.note_id.as_deref()).ok_or_else(|| {
            Failure::new(
                kanso_plugin_host::protocol::CODE_UNAVAILABLE,
                "the library can't be read just now",
            )
        })
    }

    /// The open note, when the plugin may see it.
    fn open_note(&self) -> Result<Option<String>, Failure> {
        let Some(id) = self.editor.note_id.clone() else {
            return Ok(None);
        };
        Ok(self.scope()?.open_note_visible(&id).then_some(id))
    }

    /// Answers host call `call` with `args`.
    pub fn call(&mut self, call: &str, args: Value) -> Result<Value, Failure> {
        if self.request == Request::Render {
            return Err(self.refuse(&format!("to use kanso.{call} while drawing")));
        }
        match call {
            "notes.list" => self.notes_list(&args),
            "notes.get" => self.notes_get(&args),
            "notes.create" => self.notes_create(&args),
            "notes.update" => self.notes_update(&args),
            "notes.pluginData.get" => self.data_get(&args),
            "notes.pluginData.set" => self.data_set(&args),
            "attachments.get" => self.attachment(&args),
            "editor.noteId" => self.editor_note(),
            "editor.selection" => self.editor_selection(),
            "editor.replaceSelection" => self.replace_selection(&args),
            "ui.panel" => self.panel(&args),
            "settings.get" => self.settings_get(),
            "settings.set" => self.settings_set(&args),
            "secrets.get" => self.secret(&args),
            "clipboard.history" => self.clipboard_history(),
            "clipboard.write" => self.clipboard_write(&args),
            "fetch" => self.fetch(&args),
            _ => Err(self.refuse(&format!("to call {call}"))),
        }
    }

    fn needs_reads(&mut self) -> Result<(), Failure> {
        let grant = self.grant();
        if grant.notes.reads() || grant.editor.contains(&EditorPermission::NoteRead) {
            Ok(())
        } else {
            Err(self.refuse("to read notes"))
        }
    }

    fn notes_list(&mut self, args: &Value) -> Result<Value, Failure> {
        self.needs_reads()?;
        let scope = self.scope()?;
        let (notes, cursor) = scope.list(&ListQuery::from_args(args));
        let notes: Vec<Value> = notes.iter().map(|n| scope.summary_json(n)).collect();
        Ok(json!({ "notes": notes, "cursor": cursor }))
    }

    fn notes_get(&mut self, args: &Value) -> Result<Value, Failure> {
        self.needs_reads()?;
        let id = text_arg(args, "id")?;
        let scope = self.scope()?;
        let note = scope.readable(id).ok_or_else(Failure::not_found)?;
        Ok(scope.note_json(&note))
    }

    fn notes_create(&mut self, args: &Value) -> Result<Value, Failure> {
        if !self.grant().notes.create || !self.grant().notes.writes() {
            return Err(self.refuse("to make notes"));
        }
        let book_id = text_arg(args, "bookId")?.to_owned();
        let title = args
            .get("title")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        let body = args
            .get("body")
            .and_then(Value::as_str)
            .unwrap_or("")
            .to_owned();
        if title.chars().count() > MAX_TITLE_LEN || body.chars().count() > MAX_BODY_LEN {
            return Err(invalid("the note is too long"));
        }
        if !self.scope()?.can_create_in(&book_id) {
            return Err(self.refuse("to make notes in that notebook"));
        }
        self.out.batch.stage_create(PluginCreate {
            book_id,
            title,
            body,
        })?;
        Ok(json!({ "staged": true }))
    }

    /// The note `id` the plugin may change, at `base_rev`, or why not.
    fn writable_at(&mut self, id: &str, base_rev: Option<&str>) -> Result<String, Failure> {
        if !self.grant().notes.writes() {
            return Err(self.refuse("to change notes"));
        }
        let scope = self.scope()?;
        let readable = scope.readable(id).ok_or_else(Failure::not_found)?;
        if scope.writable(id).is_none() {
            drop(scope);
            return Err(self.refuse("to change notes in that notebook"));
        }
        let rev = readable.rev.unwrap_or_default();
        if let Some(base) = base_rev
            && base != rev
        {
            return Err(Failure::new(
                CODE_CONFLICT,
                "the note changed since it was read",
            ));
        }
        Ok(rev)
    }

    fn notes_update(&mut self, args: &Value) -> Result<Value, Failure> {
        let id = text_arg(args, "id")?.to_owned();
        let base_rev = text_arg(args, "baseRev")
            .map_err(|_| invalid("baseRev is required"))?
            .to_owned();
        let mut edit = edit_fields(args.get("fields").unwrap_or(&Value::Null))?;
        self.writable_at(&id, Some(&base_rev))?;
        edit.id = id.clone();
        edit.base_rev = base_rev;
        self.out.batch.stage_edit(edit)?;
        Ok(json!({ "id": id, "staged": true }))
    }

    fn data_get(&mut self, args: &Value) -> Result<Value, Failure> {
        self.needs_reads()?;
        let id = text_arg(args, "id")?;
        let scope = self.scope()?;
        scope.readable(id).ok_or_else(Failure::not_found)?;
        let doc = self
            .store
            .get(id)
            .ok()
            .flatten()
            .ok_or_else(Failure::not_found)?;
        Ok(plugin_data(&doc.fields, self.plugin.id())
            .cloned()
            .unwrap_or(Value::Null))
    }

    fn data_set(&mut self, args: &Value) -> Result<Value, Failure> {
        let id = text_arg(args, "id")?.to_owned();
        let value = args.get("value").cloned().unwrap_or(Value::Null);
        if value.to_string().len() > MAX_PLUGIN_DATA {
            return Err(Failure::new(
                CODE_LIMIT,
                "a note's plugin data is at most 32 KB",
            ));
        }
        let rev = self.writable_at(&id, None)?;
        let base_rev = self
            .out
            .batch
            .edits
            .iter()
            .find(|e| e.id == id)
            .map_or(rev, |e| e.base_rev.clone());
        self.out.batch.stage_edit(PluginEdit {
            id: id.clone(),
            base_rev,
            data: Some((!value.is_null()).then_some(value)),
            ..PluginEdit::default()
        })?;
        Ok(json!({ "id": id, "staged": true }))
    }

    fn attachment(&mut self, args: &Value) -> Result<Value, Failure> {
        self.needs_reads()?;
        let file = text_arg(args, "fileId")?;
        let scope = self.scope()?;
        let (content_type, data) = scope.attachment(file).ok_or_else(Failure::not_found)?;
        if data.len() > MAX_ATTACHMENT_BYTES {
            return Err(Failure::new(CODE_LIMIT, "the file is larger than 4 MB"));
        }
        Ok(json!({
            "contentType": content_type,
            "size": data.len(),
            "data": base64::engine::general_purpose::STANDARD.encode(data),
        }))
    }

    fn editor_note(&mut self) -> Result<Value, Failure> {
        if !self.grant().editor.contains(&EditorPermission::NoteRead) {
            return Err(self.refuse("to read the open note"));
        }
        Ok(self.open_note()?.map_or(Value::Null, Value::from))
    }

    fn editor_selection(&mut self) -> Result<Value, Failure> {
        if !self
            .grant()
            .editor
            .contains(&EditorPermission::SelectionRead)
        {
            return Err(self.refuse("to read the selection"));
        }
        if self.open_note()?.is_none() {
            return Ok(Value::Null);
        }
        Ok(match &self.editor.selection {
            Some(s) => json!({ "text": s.text, "start": s.start, "end": s.end }),
            None => Value::Null,
        })
    }

    fn replace_selection(&mut self, args: &Value) -> Result<Value, Failure> {
        if !self
            .grant()
            .editor
            .contains(&EditorPermission::SelectionReplace)
        {
            return Err(self.refuse("to change the selection"));
        }
        let text = text_arg(args, "text")?.to_owned();
        if text.len() > MAX_REPLACEMENT_BYTES {
            return Err(Failure::new(CODE_LIMIT, "the text is too long"));
        }
        let note_id = self.open_note()?.ok_or_else(Failure::not_found)?;
        let selection = self
            .editor
            .selection
            .clone()
            .ok_or_else(|| invalid("nothing is selected"))?;
        if let Some(given) = args.get("selection") {
            let at = |key: &str| given.get(key).and_then(Value::as_u64).map(|n| n as usize);
            if at("start") != Some(selection.start) || at("end") != Some(selection.end) {
                return Err(Failure::new(CODE_CONFLICT, "the selection changed"));
            }
        }
        self.out.selection_edit = Some(SelectionEdit {
            note_id,
            start: selection.start,
            end: selection.end,
            text,
        });
        Ok(json!({ "staged": true }))
    }

    fn panel(&mut self, args: &Value) -> Result<Value, Failure> {
        if !self.grant().panel {
            return Err(self.refuse("to show a panel"));
        }
        let tree = crate::panel::check(args.get("tree").unwrap_or(&Value::Null))
            .map_err(|e| invalid(e.to_string()))?;
        let scope = self.scope()?;
        for image in crate::panel::images(&tree) {
            if scope.attachment(image).is_none() {
                return Err(Failure::not_found());
            }
        }
        drop(scope);
        self.out.panel = Some(tree);
        Ok(Value::Null)
    }

    fn settings_get(&mut self) -> Result<Value, Failure> {
        Ok(crate::install::read_settings(self.plugin_dir))
    }

    fn settings_set(&mut self, args: &Value) -> Result<Value, Failure> {
        let value = args.get("value").cloned().unwrap_or(Value::Null);
        if value.to_string().len() > MAX_SETTINGS_BYTES {
            return Err(Failure::new(CODE_LIMIT, "settings are at most 64 KB"));
        }
        crate::install::write_settings(self.plugin_dir, &value).map_err(|e| {
            Failure::new(kanso_plugin_host::protocol::CODE_UNAVAILABLE, e.to_string())
        })?;
        Ok(Value::Null)
    }

    fn secret(&mut self, args: &Value) -> Result<Value, Failure> {
        let name = text_arg(args, "name")?.to_owned();
        if !self.grant().secrets.contains(&name) {
            return Err(self.refuse(&format!("the token {name}")));
        }
        Ok(self
            .secrets
            .get(self.plugin.id(), &name)
            .map_or(Value::Null, Value::from))
    }

    fn clipboard_history(&mut self) -> Result<Value, Failure> {
        if !self
            .grant()
            .clipboard
            .contains(&ClipboardPermission::HistoryRead)
        {
            return Err(self.refuse("to read the clipboard history"));
        }
        Ok(Value::Array(
            self.clipboard
                .iter()
                .map(|clip| json!({ "text": clip.text, "copiedAt": clip.copied_ms }))
                .collect(),
        ))
    }

    fn clipboard_write(&mut self, args: &Value) -> Result<Value, Failure> {
        if !self.grant().clipboard.contains(&ClipboardPermission::Write) {
            return Err(self.refuse("to put text on the clipboard"));
        }
        let text = text_arg(args, "text")?.to_owned();
        if text.len() > MAX_CLIPBOARD_BYTES {
            return Err(Failure::new(CODE_LIMIT, "the text is too long"));
        }
        self.out.clipboard_write = Some(text);
        Ok(Value::Null)
    }

    fn fetch(&mut self, args: &Value) -> Result<Value, Failure> {
        if self.grant().network.is_empty() {
            return Err(self.refuse("to use the network"));
        }
        let hosts = self.grant().network.clone();
        let id = self.plugin.id().to_owned();
        self.network.fetch(&id, &hosts, args)
    }
}

/// The values of `node`'s inputs, toggles and selects as the panel holds
/// them, for a press. Pure.
pub fn press_values(node: &Node, held: &Map<String, Value>) -> Map<String, Value> {
    crate::panel::inputs(node)
        .into_iter()
        .filter_map(|id| held.get(id).map(|v| (id.to_owned(), v.clone())))
        .collect()
}
