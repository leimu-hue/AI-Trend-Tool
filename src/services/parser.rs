use async_trait::async_trait;
use chrono::NaiveDateTime;
use sqlx::SqlitePool;
use std::sync::Arc;
use tokio::sync::Semaphore;

use crate::config::ParserConfig;
use crate::db;
use crate::models::source::DataSource;
use crate::pipeline::{Pipeline, PipelineEvent};

/// A parsed article extracted from a feed
#[derive(Debug, Clone)]
pub struct ParsedArticle {
    pub link: String,
    pub title: String,
    pub summary: String,
    pub content: String,
    pub published_at: Option<NaiveDateTime>,
}

/// Extensible parser trait for different feed types
#[async_trait]
pub trait Parser: Send + Sync {
    /// Fetch and parse a feed, returning extracted articles.
    /// Returns an error if the feed cannot be fetched or parsed.
    async fn fetch_and_parse(
        &self,
        source: &DataSource,
    ) -> Result<Vec<ParsedArticle>, Box<dyn std::error::Error + Send + Sync>>;
}

/// RSS/Atom feed parser using `feed-rs`
pub struct RssParser {
    client: reqwest::Client,
}

impl RssParser {
    pub fn new(config: &ParserConfig) -> Self {
        let client = reqwest::Client::builder()
            .user_agent(&config.default_user_agent)
            .timeout(std::time::Duration::from_secs(
                config.default_timeout_seconds,
            ))
            .build()
            .expect("Failed to build reqwest client");
        Self { client }
    }
}

#[async_trait]
impl Parser for RssParser {
    async fn fetch_and_parse(
        &self,
        source: &DataSource,
    ) -> Result<Vec<ParsedArticle>, Box<dyn std::error::Error + Send + Sync>> {
        let response = self.client.get(&source.url).send().await?;
        let body = response.bytes().await?;
        let feed = feed_rs::parser::parse(&body[..])?;

        let articles: Vec<ParsedArticle> = feed
            .entries
            .into_iter()
            .filter_map(|entry| {
                let link = entry.links.first().map(|l| l.href.clone())?;
                let title = entry.title.map(|t| t.content).unwrap_or_default();
                let summary = entry.summary.map(|s| s.content).unwrap_or_default();
                let content = entry.content.and_then(|c| c.body).unwrap_or_default();
                let published_at = entry
                    .published
                    .or(entry.updated)
                    .and_then(|d| chrono::DateTime::from_timestamp(d.timestamp(), 0))
                    .map(|dt| dt.naive_utc());

                Some(ParsedArticle {
                    link,
                    title,
                    summary,
                    content,
                    published_at,
                })
            })
            .collect();

        Ok(articles)
    }
}

/// Background parser loop.
///
/// Uses `tokio::select!` to listen for cancellation while periodically
/// querying due sources at the configured `interval_seconds`.
/// Spawns concurrent fetch tasks limited by `config.parser.max_concurrent_fetches`.
/// Uses `JoinSet` to track spawned tasks and wait for them on shutdown.
pub async fn start_parser_loop(pool: SqlitePool, config: ParserConfig, pipeline: Pipeline) {
    let parser = Arc::new(RssParser::new(&config));
    let semaphore = Arc::new(Semaphore::new(config.max_concurrent_fetches));
    let mut interval =
        tokio::time::interval(std::time::Duration::from_secs(config.interval_seconds));
    let mut joinset = tokio::task::JoinSet::new();

    loop {
        tokio::select! {
            _ = pipeline.cancel.cancelled() => {
                tracing::info!("Parser: shutting down gracefully, waiting for {} in-flight task(s)", joinset.len());
                while let Some(result) = joinset.join_next().await {
                    if let Err(e) = result {
                        tracing::error!("Parser: in-flight task panicked: {}", e);
                    }
                }
                tracing::info!("Parser: all in-flight tasks complete");
                break;
            }
            _ = interval.tick() => {}
        }

        let due_sources = match db::source::list_due_sources(&pool).await {
            Ok(sources) => sources,
            Err(e) => {
                tracing::error!("Parser: failed to query due sources: {}", e);
                continue;
            }
        };

        if due_sources.is_empty() {
            continue;
        }

        tracing::info!("Parser: {} source(s) due for fetching", due_sources.len());

        for source in due_sources {
            let pool = pool.clone();
            let parser = Arc::clone(&parser);
            let permit = Arc::clone(&semaphore);
            let pipeline = pipeline.clone();

            joinset.spawn(async move {
                let _permit = permit.acquire().await;

                let result = parser.fetch_and_parse(&source).await;

                match result {
                    Ok(articles) => {
                        let mut inserted = 0;
                        let mut skipped = 0;
                        for article in &articles {
                            match db::article::insert_article(
                                &pool,
                                source.id,
                                &article.link,
                                &article.title,
                                &article.summary,
                                &article.content,
                                article.published_at,
                            )
                            .await
                            {
                                Ok(Some(_)) => inserted += 1,
                                Ok(None) => skipped += 1,
                                Err(e) => {
                                    tracing::error!(
                                        "Parser: failed to insert article '{}': {}",
                                        article.link,
                                        e
                                    );
                                }
                            }
                        }
                        // Update last_fetched_at on success
                        if let Err(e) =
                            db::source::update_source_last_fetched(&pool, source.id).await
                        {
                            tracing::error!(
                                "Parser: failed to update last_fetched for source {}: {}",
                                source.id,
                                e
                            );
                        }
                        // Notify Filter that new articles are available
                        if inserted > 0 {
                            let _ = pipeline.articles_ready_tx.try_send(PipelineEvent::NewData);
                        }
                        tracing::info!(
                            "Parser: source {} '{}' — {} inserted, {} skipped (dup)",
                            source.id,
                            source.name,
                            inserted,
                            skipped
                        );
                    }
                    Err(e) => {
                        tracing::error!(
                            "Parser: failed to fetch source {} '{}': {}",
                            source.id,
                            source.name,
                            e
                        );
                        // Still update last_fetched so we don't retry immediately
                        let _ = db::source::update_source_last_fetched(&pool, source.id).await;
                    }
                }
            });
        }
    }
}
