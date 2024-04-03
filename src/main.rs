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
static DB_FILE: &str = "/tmp/russet-db.sqlite";

pub type Err = Box<dyn Error>;
pub type Result<T> = std::result::Result<T, Err>;

fn main() -> Result<()> {
	println!("================================================================================");
	let async_util = Arc::new(AsyncUtil::new());
	let db = SqlDatabase::new(Path::new(DB_FILE), async_util.clone())?;
	let reader = Box::new(AtomFeedReader::new());
	let mut domain_service = RussetDomainService::new(db, vec![reader], async_util.clone());
	domain_service.update_feeds()?;
	println!("================================================================================");
	Ok(())
}

