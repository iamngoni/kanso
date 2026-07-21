/// A global label that can attach to notes.
#[derive(Debug, Clone)]
pub struct Tag {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
}
impl_sqlite_from_row!(Tag { id, name, color });
