use clap::{ Parser, Subcommand};
use merge::Merge;
use serde::Deserialize;
use std::num::ParseIntError;
use std::time::Duration;

#[derive(Deserialize, Merge, Parser)]
#[command(version = crate::VERSION, about, long_about = None)]
#[serde(default)]
pub struct Config {
	/// Command
	#[command(subcommand)]
	pub command: Option<Command>,

	/// Config file
	#[arg(short, long, value_name = "FILE")]
	#[serde(skip)] // We're not going to recursively load config files
	pub config_file: Option<String>,

	/// Database file
	#[arg(short, long, value_name = "FILE")]
	pub db_file: Option<String>,

	/// Listen address
	#[arg(short, long, value_name = "ADDRESS")]
	pub listen_address: Option<String>,

	/// Pepper for password hashing
	///
	/// (Not exposed on the CLI; let's not enoucrage putting secrets in
	/// commandlines or shell histories.)
	pub pepper: Option<String>,

	/// Duration between feed checks, in seconds
	#[arg(
		short,
		long,
		value_name = "SECONDS",
		value_parser = |arg: &str| Ok::<Duration, ParseIntError>(
			Duration::from_secs(arg.parse()?)
		)
	)]
	pub feed_check_interval: Option<Duration>,
}
impl Default for Config {
	fn default() -> Self {
		Config {
			command: Some(Command::Run),
			config_file: None,
			db_file: Some("/tmp/russet-db.sqlite".to_string()),
			listen_address: Some("127.0.0.1:9892".to_string()),
			pepper: Some("IzvoEPMQIi82NSXTz7cZ".to_string()),
			feed_check_interval: Some(Duration::from_secs(3_600)),
		}
	}
}
impl std::fmt::Debug for Config {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Config")
			.field("db_file", &self.db_file)
			.field("config_file", &self.config_file)
			.field("listen_address", &self.listen_address)
			.field("pepper", &"<redacted>")
			.field("feed_check_interval", &self.feed_check_interval.map(|duration| duration.as_secs()))
			.finish()
	}
}

#[derive(Debug, Deserialize, Subcommand)]
pub enum Command {
	/// Run the Russet server
	Run,

	/// Add a user
	AddUser {
		user_name: String,
		password: Option<String>,
	},

	/// Reset a user's password
	SetUserPassword {
		user_name: String,
		password: Option<String>,
	},

	/// Delete a user
	DeleteUser {
		user_name: String,
	},

	/// Delete all sessions for a user
	DeleteSessions {
		user_name: String,
	},

	/// Add a feed by URL
	AddFeed {
		url: String,
	},

	/// Remove a feed by URL
	RemoveFeed {
		url: String,
	},
}
