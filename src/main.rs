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
	let async_util = Arc::new(AsyncUtil::new());
	let mut db = SqlDatabase::new(Path::new(DB_FILE), async_util.clone())?;
	let url = Url::parse(FEED_URL).unwrap();
	let reader = AtomFeedReader { };
/*
	let (feed, entries) = update(&url, &reader, &mut db).await?;
	println!("{:?}", feed);
	for entry in entries {
		println!("\t{:?}", entry);
	}
	println!("================================================================================");
	db.get_feeds().await.for_each(|feed| {
		println!("{:?}", feed);
		futures::future::ready(())
	} ).await;
*/
	Ok(())
}

/*
async fn update<F, P>(url: &Url, reader: &F, db: &mut P) -> Result<(persistence::model::Feed, Vec<persistence::model::Entry>)>
where F: RussetFeedReader, P: RussetPersistanceLayer {
	let fetch_index = db.get_and_increment_fetch_index().await?;
	let stored_feed_future = db.get_feed_by_url(url);
	let download_feed_future = reader.load_feed(url);
	let download_feed = download_feed_future.await?;
	let stored_feed = match stored_feed_future.await {
		Ok(feed) => feed,
		Err(_) => {
			let feed = persistence::model::Feed {
				id: Ulid::new(),
				title: download_feed.title.clone(),
				url: url.clone(),
			};
			db.add_feed(&feed).await?;
			feed
		}
	};
	let stored_entries = download_feed.entries.into_iter().map(|entry| {
		persistence::model::Entry {
			id: Ulid::new(),
			internal_id: entry.internal_id,
			fetch_index,
			article_date: entry.article_date,
			title: entry.title,
			url: entry.url,
		}
	} ).collect();

	Ok((stored_feed, stored_entries))
}
*/
