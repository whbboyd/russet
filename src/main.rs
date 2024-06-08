extern crate atom_syndication;
extern crate reqwest;
extern crate rss;
extern crate sqlx;
extern crate tokio;

mod conf;
mod domain;
mod feed;
mod http;
mod persistence;
mod server;
mod model;

use clap::Parser;
use crate::conf::{ Command, Config };
use crate::domain::RussetDomainService;
use crate::feed::atom::AtomFeedReader;
use crate::feed::rss::RssFeedReader;
use crate::feed::RussetFeedReader;
use crate::persistence::sql::SqlDatabase;
use crate::server::start;
use merge::Merge;
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
static REPO_URL: &str = "https://git.sr.ht/~whbboyd/russet";
static APP_NAME: &str = "Russet";
static VERSION: &str = env!("CARGO_PKG_VERSION");

pub type Err = Box<dyn Error + Send + Sync + 'static>;
pub type Result<T> = std::result::Result<T, Err>;

#[tokio::main]
async fn main() -> Result<()> {
	init_tracing()?;

	// Hierarchy of configs
	let config = {
		// Commandline flags override all
		let mut config = Config::parse();

		// If present, load config file
		if let Some(&ref config_file) = config.config_file.as_ref() {
				let s = read_to_string(config_file)?;
				let file_config = toml::from_str(&s)?;
				config.merge(file_config);
		}

		// Finally, load defaults
		config.merge(Config::default());

		config
	};

	let command = config.command.expect("No command");
	let db_file = config.db_file.expect("No db_file");
	let listen_address = config.listen_address.expect("No listen_address");
	let pepper = config.pepper.expect("No pepper");
	let feed_check_interval =
		config.feed_check_interval.expect("No feed_check_interval");
	let disable_logins = config.disable_logins.expect("No disable_logins");
	let global_concurrent_limit = config
		.rate_limiting
		.global_concurrent_limit
		.expect("No global_concurrent_limit");
	let login_concurrent_limit = config
		.rate_limiting
		.login_concurrent_limit
		.expect("No login_concurrent_limit");

	let db = SqlDatabase::new(Path::new(&db_file)).await?;
	let readers: Vec<Box<dyn RussetFeedReader>> = vec![
		Box::new(RssFeedReader::new()),
		Box::new(AtomFeedReader::new()),
	];
	let domain_service = Arc::new(RussetDomainService::new(
		db,
		readers,
		pepper.as_bytes().to_vec(),
		feed_check_interval,
		disable_logins,
	)?);

	match command {
		Command::Run => start(
				domain_service,
				listen_address,
				global_concurrent_limit,
				login_concurrent_limit
			)
			.await?,
		Command::AddUser { user_name, password, user_type } => {
			info!("Adding user {user_name}…");
			let plaintext_password = match password {
				Some(password) => password,
				None => prompt_password(format!("Enter password for {user_name}: "))?,
			};
			let user_type = match user_type {
				Some(user_type) => user_type,
				None => {
					print!("Enter user type (\"Member\" or \"Sysop\"): ");
					use std::io::Write;
					std::io::stdout().flush()?;
					let mut user_type_str = String::new();
					std::io::stdin().read_line(&mut user_type_str)?;
					user_type_str.trim_end().to_string().try_into()?
				}
			};
			domain_service.add_user(&user_name, &plaintext_password, user_type).await?;
		},
		Command::SetUserPassword { user_name, password } => {
			info!("Setting password for user {user_name}…");
			let plaintext_password = match password {
				Some(password) => password,
				None => prompt_password(format!("Enter password for {user_name}: "))?,
			};
			domain_service
				.set_user_password(&user_name, &plaintext_password)
				.await?;
		},
		Command::DeleteUser { user_name } => {
			info!("Deleting user {user_name}…");
			domain_service.delete_user(&user_name).await?;
		}
		Command::DeleteSessions { user_name } => {
			info!("Deleting sessions for {user_name}…");
			domain_service.delete_user_sessions(&user_name).await?;
		}
		_ => { warn!("Not yet implemented") },
	}
	info!("Done!");
	Ok(())
}

fn init_tracing() -> Result<()> {
	let filter = EnvFilter::builder()
		.with_default_directive(LevelFilter::INFO.into())
		.from_env()?;
	let subscriber = tracing_subscriber::fmt::layer();
//`		.with_span_events(FmtSpan::NEW | FmtSpan::CLOSE);
	tracing_subscriber::registry()
		.with(filter)
		.with(subscriber)
		.init();
	Ok(())
}
