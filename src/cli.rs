use clap::{ Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version = crate::VERSION, about, long_about = None)]
pub struct Cli {
	/// Command
	#[command(subcommand)]
	pub command: Option<Command>,

	/// Config file
	#[arg(short, long, value_name = "FILE")]
	pub config_file: Option<String>,

	/// Database file
	#[arg(short, long, value_name = "FILE")]
	pub db_file: Option<String>,

	/// Listen address
	#[arg(short, long, value_name = "ADDRESS")]
	pub listen_address: Option<String>,
}

#[derive(Debug, Subcommand)]
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
