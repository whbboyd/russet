extern crate atom_syndication;
extern crate reqwest;
extern crate rss;
extern crate sqlx;
extern crate tokio;

mod domain;
mod feed;
mod http;
mod persistence;

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
static LISTEN: &str = "127.0.0.1:9892";

pub type Err = Box<dyn Error>;
pub type Result<T> = std::result::Result<T, Err>;

#[tokio::main]
async fn main() -> Result<()> {
	println!("================================================================================");
	let db = SqlDatabase::new(Path::new(DB_FILE)).await?;
	let reader = Box::new(AtomFeedReader::new());
	let mut domain_service = RussetDomainService::new(db, vec![reader], PEPPER.as_bytes().to_vec())?;
	let url = reqwest::Url::parse(FEED_URL).unwrap();
	domain_service.add_feed(&url).await?;
	println!("================================================================================");
	let app_state = http::AppState { hello: "Hello, state!".to_string(), domain_service: Arc::new(domain_service) };
	let routes = http::russet_router()
		.with_state(app_state);
	let listener = tokio::net::TcpListener::bind(LISTEN).await?;
	axum::serve(listener, routes).await?;
	println!("================================================================================");
	Ok(())
}

