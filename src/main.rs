extern crate atom_syndication;
extern crate reqwest;
extern crate rss;
extern crate sqlx;
extern crate tokio;

pub mod feed;
pub mod persistence;
mod async_util;

use async_util::AsyncUtil;
use feed::atom::AtomFeedReader;
use feed::RussetFeedReader;
use persistence::RussetPersistanceLayer;
use persistence::sql::SqlDatabase;
use reqwest::Url;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use ulid::Ulid;

// TODO: hardcoded config for now
static FEED_URL: &str = "https://whbboyd.com/feeds/atom.xml";
static DB_FILE: &str = "/tmp/russet-db.sqlite";

pub type Err = Box<dyn Error>;
pub type Result<T> = std::result::Result<T, Err>;

fn main() -> Result<()> {
	println!("================================================================================");
	let async_util = Arc::new(AsyncUtil::new());
	let mut db = SqlDatabase::new(Path::new(DB_FILE), async_util.clone())?;
	let url = Url::parse(FEED_URL).unwrap();
	let reader = AtomFeedReader::new(async_util.clone());
	let reader_feed = reader.load_feed(&url)?;
	let storage_feed = db.get_feed_by_url(&url).or_else(|_| {
		let feed = persistence::model::Feed {
			id: Ulid::new(),
			title: reader_feed.title,
			url: url.clone(),
		};
		db.add_feed(&feed)?;
		Ok::<persistence::model::Feed, Err>(feed)
	} )?;
	println!("Updating \"{}\" ({})…", storage_feed.title, storage_feed.id.to_string());
	let fetch_index = db.get_and_increment_fetch_index()?;
	let storage_entries = db.get_entries_for_feed(&storage_feed.id).into_iter().collect::<Vec<Result<persistence::model::Entry>>>();
	// TODO: Make this not quadratic
	let new_entries = reader_feed.entries.into_iter()
		.filter(|entry| {
			for s in storage_entries.as_slice() {
				if s.as_ref().map(|e| e.internal_id == entry.internal_id).unwrap_or(false) {
					return false
				}
			};
			true
		} )
		.map(|entry| {
			persistence::model::Entry {
				id: Ulid::new(),
				internal_id: entry.internal_id,
				fetch_index,
				article_date: entry.article_date,
				title: entry.title,
				url: entry.url,
			}
		} )
		.collect::<Vec<persistence::model::Entry>>();
	println!("{} new entries", new_entries.len());
	for e in new_entries.as_slice() {
		println!("\t{} ({})", e.title, e.id.to_string());
		db.add_entry(e, &storage_feed.id)?;
	}
	println!("================================================================================");
	Ok(())
}

