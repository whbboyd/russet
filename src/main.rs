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
use feed::rss::RssFeedReader;
use feed::RussetFeedReader;
use persistence::sql::SqlDatabase;
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tracing::{ error, info };
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
//use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// TODO: hardcoded config for now
static DB_FILE: &str = "/tmp/russet-db.sqlite";
static PEPPER: &str = "IzvoEPMQIi82NSXTz7cZ";
static LISTEN: &str = "127.0.0.1:9892";

pub type Err = Box<dyn Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, Err>;

#[tokio::main]
async fn main() -> Result<()> {
	init_tracing();
	info!("Starting Russet…");
	let db = SqlDatabase::new(Path::new(DB_FILE)).await?;
	let readers: Vec<Box<dyn RussetFeedReader>> = vec![
		Box::new(RssFeedReader::new()),
		Box::new(AtomFeedReader::new()),
	];
	let domain_service = Arc::new(RussetDomainService::new(db, readers, PEPPER.as_bytes().to_vec())?);

	info!("Setup complete, initializing…");
	// TODO: Initialize with hard-coded user
	let _swallowed = domain_service.add_user("admin", "swordfish").await;

	// Start the feed update coroutine
	let update_service = domain_service.clone();
	tokio::spawn(async move {
		loop {
			info!("Updating feeds");
			if let Err(err) = update_service.update_feeds().await {
				error!("Error updating feeds: {}", err);
			}
			tokio::time::sleep(Duration::from_secs(/*FIXME*/3_600)).await;
		}
	} );

	// Setup for Axum
	let app_state = http::AppState { domain_service: domain_service.clone() };
	let routes = http::russet_router()
		.with_state(app_state);
	let listener = tokio::net::TcpListener::bind(LISTEN).await?;
	info!("Initialization complete, serving requests!");
	info!("Listening on {LISTEN}…");
	axum::serve(listener, routes).await?;
	info!("Exiting Russet…");
	Ok(())
}

fn init_tracing() {
	let filter = EnvFilter::builder()
		.with_default_directive(LevelFilter::INFO.into())
		.from_env()
		.unwrap();
	let subscriber = tracing_subscriber::fmt::layer();
//`		.with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);
	tracing_subscriber::registry()
		.with(filter)
		.with(subscriber)
		.init();
}
