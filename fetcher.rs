use crate::db::Database;
use crate::models::Feed;
use crate::parser::parse_feed_content;
use anyhow::Result;
use futures::future::join_all;
use reqwest::header::{IF_MODIFIED_SINCE, IF_NONE_MATCH};
use reqwest::Client;
use std::sync::Arc;
use tokio::sync::Semaphore;

pub struct Fetcher {
    client: Client,
    db: Database,
    concurrency_limit: Arc<Semaphore>,
}

impl Fetcher {
    pub fn new(db: Database, max_concurrent: usize) -> Self {
        let client = Client::builder()
            .timeout(std::time::Duration::from_secs(10))
            .user_agent("Rust-RSS-Aggregator/1.0")
            .build()
            .unwrap();

        Self {
            client,
            db,
            concurrency_limit: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    pub async fn fetch_feed(&self, feed: Feed) -> Result<usize> {
        let _permit = self.concurrency_limit.acquire().await?;

        let mut req = self.client.get(&feed.url);

        if let Some(ref etag) = feed.etag {
            req = req.header(IF_NONE_MATCH, etag);
        }
        if let Some(ref last_mod) = feed.last_modified {
            req = req.header(IF_MODIFIED_SINCE, last_mod);
        }

        let resp = match req.send().await {
            Ok(r) => r,
            Err(e) => {
                let _ = self.db.update_feed_metadata(feed.id, None, None, None, false);
                return Err(anyhow::anyhow!("Erro ao consultar HTTP {}: {}", feed.url, e));
            }
        };

        if resp.status() == reqwest::StatusCode::NOT_MODIFIED {
            let _ = self.db.update_feed_metadata(
                feed.id,
                feed.title.as_deref(),
                feed.etag.as_deref(),
                feed.last_modified.as_deref(),
                true,
            );
            return Ok(0); 
        }

        if !resp.status().is_success() {
            let _ = self.db.update_feed_metadata(feed.id, None, None, None, false);
            return Err(anyhow::anyhow!("Status HTTP inválido: {}", resp.status()));
        }

        let etag = resp.headers().get("etag").and_then(|v| v.to_str().ok()).map(|s| s.to_string());
        let last_mod = resp.headers().get("last-modified").and_then(|v| v.to_str().ok()).map(|s| s.to_string());

        let bytes = resp.bytes().await?;
        let parsed = match parse_feed_content(feed.id, &bytes) {
            Ok(p) => p,
            Err(e) => {
                let _ = self.db.update_feed_metadata(feed.id, None, None, None, false);
                return Err(anyhow::anyhow!("Erro de Parsing no XML de {}: {}", feed.url, e));
            }
        };

        let mut new_count = 0;
        for article in &parsed.articles {
            if self.db.insert_article(article).unwrap_or(false) {
                new_count += 1;
            }
        }

        self.db.update_feed_metadata(
            feed.id,
            parsed.title.as_deref(),
            etag.as_deref(),
            last_mod.as_deref(),
            true,
        )?;

        Ok(new_count)
    }

    pub async fn fetch_all(&self) -> Result<usize> {
        let feeds = self.db.list_feeds()?;
        let mut tasks = Vec::new();

        for feed in feeds {
            tasks.push(async move {
                self.fetch_feed(feed).await
            });
        }

        let results = join_all(tasks).await;
        let total_new: usize = results.into_iter().map(|r| r.unwrap_or(0)).sum();
        Ok(total_new)
    }
}