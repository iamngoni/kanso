#[derive(Debug, Clone)]
pub struct Share {
    pub id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl_sqlite_from_row!(Share {
    id,
    resource_type,
    resource_id,
    created_at,
    updated_at,
});

#[derive(Debug, Clone)]
pub struct ShareMember {
    pub id: String,
    pub share_id: String,
    pub resource_type: String,
    pub resource_id: String,
    pub email: String,
    pub role: String,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

impl_sqlite_from_row!(ShareMember {
    id,
    share_id,
    resource_type,
    resource_id,
    email,
    role,
    status,
    created_at,
    updated_at,
});
