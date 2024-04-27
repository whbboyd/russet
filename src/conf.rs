use serde::Deserialize;

#[derive(Deserialize)]
#[serde(default)]
pub struct Config {
	pub db_file: String,
	pub listen: String,
	pub pepper: String,
}
impl Default for Config {
	fn default() -> Self {
		Config {
			db_file: "/tmp/russet-db.sqlite".to_string(),
			listen: "127.0.0.1:9892".to_string(),
			pepper: "IzvoEPMQIi82NSXTz7cZ".to_string(),
		}
	}
}

