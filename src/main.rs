extern crate atom_syndication;
extern crate reqwest;
extern crate rss;
extern crate sqlx;
extern crate tokio;

mod async_util;
mod domain;
mod feed;
mod persistence;

use async_util::AsyncUtil;
use domain::RussetDomainService;
use feed::atom::AtomFeedReader;
use persistence::sql::SqlDatabase;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;

// TODO: hardcoded config for now
static FEED_URL: &str = "https://whbboyd.com/feeds/atom.xml";
static DB_FILE: &str = "/tmp/russet-db.sqlite";
static PEPPER: &str = "IzvoEPMQIi82NSXTz7cZ";

pub type Err = Box<dyn Error>;
pub type Result<T> = std::result::Result<T, Err>;

fn main() -> Result<()> {
	println!("================================================================================");
	let async_util = Arc::new(AsyncUtil::new());
	async_util.run_blocking(|| async {
		let db = SqlDatabase::new(Path::new(DB_FILE), async_util.clone()).await?;
		let reader = Box::new(AtomFeedReader::new());
		let mut domain_service = RussetDomainService::new(db, vec![reader], PEPPER.as_bytes())?;
		let url = reqwest::Url::parse(FEED_URL).unwrap();
		domain_service.add_feed(&url).await?;
		Ok::<(), Err>(())
	} )?;
	println!("================================================================================");
	Ok(())
}

