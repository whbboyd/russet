extern crate atom_syndication;
extern crate reqwest;
extern crate rss;
extern crate sqlx;
extern crate tokio;

mod domain;
mod feed;
mod persistence;

use domain::RussetDomainService;
use feed::atom::AtomFeedReader;
use persistence::sql::SqlDatabase;
use std::error::Error;
use std::path::Path;

// TODO: hardcoded config for now
static FEED_URL: &str = "https://whbboyd.com/feeds/atom.xml";
static DB_FILE: &str = "/tmp/russet-db.sqlite";
static PEPPER: &str = "IzvoEPMQIi82NSXTz7cZ";

pub type Err = Box<dyn Error>;
pub type Result<T> = std::result::Result<T, Err>;

#[tokio::main]
async fn main() -> Result<()> {
	println!("================================================================================");
	let db = SqlDatabase::new(Path::new(DB_FILE)).await?;
	let reader = Box::new(AtomFeedReader::new());
	let mut domain_service = RussetDomainService::new(db, vec![reader], PEPPER.as_bytes())?;
	let url = reqwest::Url::parse(FEED_URL).unwrap();
	domain_service.add_feed(&url).await?;
	println!("================================================================================");
	Ok(())
}

