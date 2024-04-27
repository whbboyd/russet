extern crate atom_syndication;
extern crate reqwest;
extern crate rss;
extern crate sqlx;
extern crate tokio;

mod cli;
mod conf;
mod domain;
mod feed;
mod http;
mod persistence;
mod server;
mod model;

use clap::Parser;
use crate::cli::{ Cli, Command };
use crate::conf::Config;
use crate::domain::RussetDomainService;
use crate::feed::atom::AtomFeedReader;
use crate::feed::rss::RssFeedReader;
use crate::feed::RussetFeedReader;
use crate::persistence::sql::SqlDatabase;
use crate::server::start;
use rpassword::prompt_password;
use std::error::Error;
use std::fs::read_to_string;
use std::path::Path;
use std::sync::Arc;
use tracing::{ info, warn };
use tracing_subscriber::EnvFilter;
use tracing_subscriber::filter::LevelFilter;
//use tracing_subscriber::fmt::format::FmtSpan;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

// TODO: move off of Github
static REPO_URL: &str = "https://github.com/whbboyd/russet";
static APP_NAME: &str = "Russet";
static VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Err = Box<dyn Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, Err>;

#[tokio::main]
async fn main() -> Result<()> {
	init_tracing();

	let cli = Cli::parse();

	let config = match cli.config_file {
		Some(file_name) => {
			let s = read_to_string(file_name)?;
			toml::from_str(&s)?
		},
		None => Config::default(),
	};

	let db_file = cli.db_file.unwrap_or(config.db_file);
	let listen_address = cli.listen_address.unwrap_or(config.listen);

	let db = SqlDatabase::new(Path::new(&db_file)).await?;
	let readers: Vec<Box<dyn RussetFeedReader>> = vec![
		Box::new(RssFeedReader::new()),
		Box::new(AtomFeedReader::new()),
	];
	let domain_service = Arc::new(RussetDomainService::new(db, readers, config.pepper.as_bytes().to_vec())?);

	match cli.command {
		None | Some(Command::Run) => start(domain_service, listen_address).await?,
		Some(Command::AddUser { user_name, password }) => {
			info!("Adding user {user_name}…");
			let plaintext_password = match password {
				Some(password) => password,
				None => prompt_password(format!("Enter password for {}: ", user_name))?,
			};
			domain_service.add_user(&user_name, &plaintext_password).await?;
		},
		Some(Command::SetUserPassword { user_name, password }) => {
			info!("Setting password for user {user_name}…");
			let plaintext_password = match password {
				Some(password) => password,
				None => prompt_password(format!("Enter password for {}: ", user_name))?,
			};
			domain_service
				.set_user_password(&user_name, &plaintext_password)
				.await?;
		},
		Some(Command::DeleteUser { user_name }) => {
			info!("Deleting user {user_name}…");
			domain_service.delete_user(&user_name).await?;
		}
		Some(Command::DeleteSessions { user_name }) => {
			info!("Delete sessions for {user_name}…");
			domain_service.delete_user_sessions(&user_name).await?;
		}
		_ => { warn!("Not yet implemented") },
	}
	info!("Done!");
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
