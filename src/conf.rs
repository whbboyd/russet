use serde::Deserialize;
use std::time::Duration;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
	pub db_file: String,
	pub listen: String,
	pub pepper: String,
	pub feed_check_interval: Duration,
}
impl Default for Config {
	fn default() -> Self {
		Config {
			db_file: "/tmp/russet-db.sqlite".to_string(),
			listen: "127.0.0.1:9892".to_string(),
			pepper: "IzvoEPMQIi82NSXTz7cZ".to_string(),
			feed_check_interval: Duration::from_secs(3_600),
		}
	}
}
impl std::fmt::Debug for Config {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Config")
			.field("db_file", &self.db_file)
			.field("listen", &self.listen)
			.field("pepper", &"<redacted>")
			.field("feed_check_interval", &self.feed_check_interval.as_secs())
			.finish()
	}
}

