use crate::model::{ EntryId, FeedId, UserId, UserType, Timestamp };
use reqwest::Url;

/// Metadata for a feed, e.g. title and feed URL
#[derive(Clone, Debug)]
pub struct Feed {
	pub id: FeedId,
	pub title: String,
	pub url: Url,
}

/// Individual entry from a given feed
#[derive(Clone, Debug)]
pub struct Entry {
	pub id: EntryId,
	pub feed_id: FeedId,
	pub internal_id: String,
	pub fetch_index: u32,
	pub article_date: Timestamp,
	pub title: String,
	pub url: Option<Url>,
}

#[derive(Clone)]
pub struct PasswordHash(pub String);
impl std::fmt::Debug for PasswordHash {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_str("\"<redacted>\"")
	}
}
#[derive(Clone, Debug)]
pub struct User {
	pub id: UserId,
	pub name: String,
	pub password_hash: PasswordHash,
	pub user_type: UserType,
}

#[derive(Clone)]
pub struct SessionToken(pub String);
impl std::fmt::Debug for SessionToken {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("\"{}<redacted>\"", &self.0[..4]))
	}
}
#[derive(Clone, Debug)]
pub struct Session {
	pub token: SessionToken,
	pub user_id: UserId,
	pub expiration: Timestamp,
}

#[derive(Clone, Debug)]
pub struct UserEntry {
	pub read: Option<Timestamp>,
	pub tombstone: Option<Timestamp>,
}
