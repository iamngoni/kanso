/// A task extracted from a note body (`- [ ]` / `- [x]`).
#[derive(Debug, Clone)]
pub struct TaskItem {
    pub id: String,
    pub note_id: String,
    pub text: String,
    pub checked: i64,
}
impl_sqlite_from_row!(TaskItem {
    id,
    note_id,
    text,
    checked,
});
