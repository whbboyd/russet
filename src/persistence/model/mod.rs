use reqwest::Url;
use std::fmt::Debug;
use std::time::SystemTime;
use ulid::Ulid;

/// Metadata for a feed, e.g. title and feed URL
pub struct Feed {
	pub id: Ulid,
	pub title: String,
	pub url: Url,
}
impl Debug for Feed {
	/// Customized `Debug` implementation because `Ulid` and `Url` have very
	/// unhelpful `Debug`s.
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f,
			"Feed {{ id: {}, title: {:?}, url: \"{}\" }}",
			self.id.to_string(),
			self.title,
			self.url.to_string(),
		)
	}
}

/// Individual entry from a given feed
pub struct Entry {
	pub id: Ulid,
	pub internal_id: String,
	pub fetch_index: u32,
	pub article_date: SystemTime,
	pub title: String,
	pub url: Option<Url>,
}
impl Debug for Entry {
	/// Customized `Debug` implementation because `Ulid` and `Url` have very
	/// unhelpful `Debug`s.
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f,
			"Entry {{ id: {}, internal_id: {:?}, fetch_index: {}, article_date: {:?}, title: {:?}, url: {:?} }}",
			self.id.to_string(),
			self.internal_id,
			self.fetch_index,
			self.article_date,
			self.title,
			self.url.as_ref().map(|u| u.to_string()),
		)
	}
}

