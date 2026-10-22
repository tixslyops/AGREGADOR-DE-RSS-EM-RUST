use chrono::{DateTime, Utc};

#[derive(Debug, Clone)]
pub struct Feed {
    pub id: i64,
    pub url: String,
    pub title: Option<String>,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub last_checked: Option<DateTime<Utc>>,
    pub failure_count: i32,
}

#[derive(Debug, Clone)]
pub struct Article {
    pub id: i64,
    pub feed_id: i64,
    pub guid_hash: String,
    pub title: String,
    pub author: Option<String>,
    pub link: String,
    pub pub_date: Option<DateTime<Utc>>,
    pub description: Option<String>,
    pub is_read: bool,
    pub is_favorite: bool,
    pub fetched_at: DateTime<Utc>,
}