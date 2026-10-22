use crate::models::{Article, Feed};
use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    pub fn init(db_path: &str) -> Result<Self> {
        let conn = Connection::open(db_path)
            .with_context(|| format!("Falha ao abrir base de dados em {}", db_path))?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS feeds (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                url TEXT NOT NULL UNIQUE,
                title TEXT,
                etag TEXT,
                last_modified TEXT,
                last_checked TEXT,
                failure_count INTEGER DEFAULT 0
            );

            CREATE TABLE IF NOT EXISTS articles (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                feed_id INTEGER NOT NULL,
                guid_hash TEXT NOT NULL UNIQUE,
                title TEXT NOT NULL,
                author TEXT,
                link TEXT NOT NULL,
                pub_date TEXT,
                description TEXT,
                is_read INTEGER DEFAULT 0,
                is_favorite INTEGER DEFAULT 0,
                fetched_at TEXT NOT NULL,
                FOREIGN KEY (feed_id) REFERENCES feeds (id) ON DELETE CASCADE
            );
            ",
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn add_feed(&self, url: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO feeds (url) VALUES (?1) ON CONFLICT(url) DO NOTHING",
            params![url],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn remove_feed(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM articles WHERE feed_id = ?1", params![id])?;
        conn.execute("DELETE FROM feeds WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_feeds(&self) -> Result<Vec<Feed>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, url, title, etag, last_modified, last_checked, failure_count FROM feeds",
        )?;

        let feed_iter = stmt.query_map([], |row| {
            let last_checked_str: Option<String> = row.get(5)?;
            let last_checked = last_checked_str
                .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&Utc));

            Ok(Feed {
                id: row.get(0)?,
                url: row.get(1)?,
                title: row.get(2)?,
                etag: row.get(3)?,
                last_modified: row.get(4)?,
                last_checked,
                failure_count: row.get(6)?,
            })
        })?;

        let mut feeds = Vec::new();
        for feed in feed_iter {
            feeds.push(feed?);
        }
        Ok(feeds)
    }

    pub fn update_feed_metadata(
        &self,
        id: i64,
        title: Option<&str>,
        etag: Option<&str>,
        last_modified: Option<&str>,
        success: bool,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        let now = Utc::now().to_rfc3339();

        if success {
            conn.execute(
                "UPDATE feeds SET title = COALESCE(?1, title), etag = ?2, last_modified = ?3, last_checked = ?4, failure_count = 0 WHERE id = ?5",
                params![title, etag, last_modified, now, id],
            )?;
        } else {
            conn.execute(
                "UPDATE feeds SET last_checked = ?1, failure_count = failure_count + 1 WHERE id = ?2",
                params![now, id],
            )?;
        }
        Ok(())
    }

    pub fn insert_article(&self, article: &Article) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let pub_date_str = article.pub_date.map(|d| d.to_rfc3339());
        let fetched_at_str = article.fetched_at.to_rfc3339();

        let res = conn.execute(
            "INSERT INTO articles 
            (feed_id, guid_hash, title, author, link, pub_date, description, is_read, is_favorite, fetched_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)
            ON CONFLICT(guid_hash) DO NOTHING",
            params![
                article.feed_id,
                article.guid_hash,
                article.title,
                article.author,
                article.link,
                pub_date_str,
                article.description,
                article.is_read as i32,
                article.is_favorite as i32,
                fetched_at_str
            ],
        )?;

        Ok(res > 0)
    }

    pub fn get_articles(
        &self,
        unread_only: bool,
        feed_id: Option<i64>,
        search_query: Option<&str>,
    ) -> Result<Vec<Article>> {
        let conn = self.conn.lock().unwrap();
        let mut query = String::from(
            "SELECT id, feed_id, guid_hash, title, author, link, pub_date, description, is_read, is_favorite, fetched_at FROM articles WHERE 1=1"
        );

        if unread_only {
            query.push_str(" AND is_read = 0");
        }
        if let Some(fid) = feed_id {
            query.push_str(&format!(" AND feed_id = {}", fid));
        }
        if let Some(q) = search_query {
            let escaped = q.replace('\'', "''");
            query.push_str(&format!(
                " AND (title LIKE '%{}%' OR description LIKE '%{}%')",
                escaped, escaped
            ));
        }
        query.push_str(" ORDER BY pub_date DESC LIMIT 100");

        let mut stmt = conn.prepare(&query)?;
        let article_iter = stmt.query_map([], |row| {
            let pub_date_str: Option<String> = row.get(6)?;
            let fetched_at_str: String = row.get(10)?;

            Ok(Article {
                id: row.get(0)?,
                feed_id: row.get(1)?,
                guid_hash: row.get(2)?,
                title: row.get(3)?,
                author: row.get(4)?,
                link: row.get(5)?,
                pub_date: pub_date_str
                    .and_then(|s| DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                description: row.get(7)?,
                is_read: row.get::<_, i32>(8)? != 0,
                is_favorite: row.get::<_, i32>(9)? != 0,
                fetched_at: DateTime::parse_from_rfc3339(&fetched_at_str)
                    .unwrap()
                    .with_timezone(&Utc),
            })
        })?;

        let mut articles = Vec::new();
        for a in article_iter {
            articles.push(a?);
        }
        Ok(articles)
    }

    pub fn set_read_status(&self, article_id: i64, is_read: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE articles SET is_read = ?1 WHERE id = ?2",
            params![is_read as i32, article_id],
        )?;
        Ok(())
    }

    pub fn set_favorite_status(&self, article_id: i64, is_favorite: bool) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE articles SET is_favorite = ?1 WHERE id = ?2",
            params![is_favorite as i32, article_id],
        )?;
        Ok(())
    }
}