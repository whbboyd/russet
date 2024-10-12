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
	pub check_id: u64,
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

/// Maintaining the feed check ID is a concern of the persistence layer, so the
/// domain layer provides this version of [FeedCheck] without it for writes.
#[derive(Clone, Debug)]
pub struct WriteFeedCheck {
	pub feed_id: FeedId,
	pub check_time: Timestamp,
	pub next_check_time: Timestamp,
	pub etag: Option<String>,
}

#[derive(Clone, Debug)]
pub struct FeedCheck {
	pub id: u64,
	pub feed_id: FeedId,
	pub check_time: Timestamp,
	pub next_check_time: Timestamp,
	pub etag: Option<String>,
}
impl FeedCheck {
	pub fn from_write_feed_check(id: u64, check: WriteFeedCheck) -> FeedCheck {
		FeedCheck {
			id,
			feed_id: check.feed_id,
			check_time: check.check_time,
			next_check_time: check.next_check_time,
			etag: check.etag,
		}
	}
}
