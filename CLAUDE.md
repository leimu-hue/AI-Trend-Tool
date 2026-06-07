# CLAUDE.md

This file provides guidance to Claude Code when working with this repository.

## Project Overview

**TrendAITool** – AI hotspot monitoring system.  
Users manage RSS data sources and keywords via a React web UI. The system automatically fetches content, matches keywords (Aho-Corasick), detects trending hotspots (statistical burst detection: moving average + standard deviation over hourly buckets), and pushes alerts through DingTalk/Feishu webhooks.

## Architecture – Pipeline Pattern

Three independent background modules run on separate schedules:

1. **Parser** – Fetches RSS feeds at configured intervals, writes new articles to `articles` (deduped by `link`).
2. **Filter** – Runs every 5 min.  
   - Reads unprocessed articles (`processed_at IS NULL`), matches keywords (Aho-Corasick).  
   - Accumulates hourly bucket counts.  
   - Detects hotspots when `current_count > mean + (std_multiplier × stddev)` and `count >= min_hot_count` (over past 24 hours).  
   - Creates `hot_events` and `push_records` (status=pending).
3. **Pusher** – Runs every 10 sec.  
   - Polls `push_records` with status `pending` or retry-due.  
   - Sends webhook POST, updates status. Exponential backoff (max 3 retries).  
   - Optimistic locking prevents duplicate sends.

All modules can run together (`hotspot all`) or independently (`hotspot api|parser|filter|pusher`).

## Key Database Tables

- `api_tokens` – Bearer tokens (revocable, optional expiry)
- `data_sources` – RSS feed configs (URL, interval, JSON config)
- `articles` – Fetched articles, `processed_at` tracks filter state
- `keywords` – Keywords with sensitivity params (`std_multiplier`, `min_hot_count`)
- `hot_events` – Detected hotspots (hourly bucket stats)
- `push_channels` – Alert channel configs (webhook URL in JSON)
- `push_records` – Per-hotspot per-channel status & retry tracking (unique: `hot_event_id, channel_id`)

## Commands

```bash
# Run all modules in one process
cargo run -- --config config.toml all

# Run individual modules
cargo run -- api       # API server only
cargo run -- parser    # Parser only
cargo run -- filter    # Filter only
cargo run -- pusher    # Pusher only

# Database migrations
cargo sqlx migrate run

# Frontend
cd frontend && npm run dev      # dev server
cd frontend && npm run build    # production build

# Production backend build
cargo build --release