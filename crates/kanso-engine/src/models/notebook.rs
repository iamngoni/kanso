/// A user-facing container for notes.
#[derive(Debug, Clone)]
pub struct Notebook {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i64,
    pub created_at: i64,
    pub updated_at: i64,
    pub deleted_at: Option<i64>,
}

impl_sqlite_from_row!(Notebook {
    id,
    name,
    parent_id,
    sort_order,
    created_at,
    updated_at,
    deleted_at,
});
