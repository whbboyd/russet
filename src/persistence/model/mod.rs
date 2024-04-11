use reqwest::Url;
use std::ops::Deref;
use std::time::SystemTime;
use ulid::Ulid;

#[derive(Clone)]
pub struct FeedId(pub Ulid);
impl Deref for FeedId { type Target = Ulid; fn deref(&self) -> &Self::Target { &self.0 } }
impl std::fmt::Debug for FeedId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("\"{}\"", &self.to_string()))
	}
}
/// Metadata for a feed, e.g. title and feed URL
#[derive(Clone, Debug)]
pub struct Feed {
	pub id: FeedId,
	pub title: String,
	pub url: Url,
}

#[derive(Clone)]
pub struct EntryId(pub Ulid);
impl Deref for EntryId{ type Target = Ulid; fn deref(&self) -> &Self::Target { &self.0 } }
impl std::fmt::Debug for EntryId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("\"{}\"", &self.to_string()))
	}
}
/// Individual entry from a given feed
#[derive(Clone, Debug)]
pub struct Entry {
	pub id: EntryId,
	pub feed_id: FeedId,
	pub internal_id: String,
	pub fetch_index: u32,
	pub article_date: SystemTime,
	pub title: String,
	pub url: Option<Url>,
}

#[derive(Clone)]
pub struct UserId(pub Ulid);
impl Deref for UserId{ type Target = Ulid; fn deref(&self) -> &Self::Target { &self.0 } }
impl std::fmt::Debug for UserId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("\"{}\"", &self.to_string()))
	}
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
	pub expiration: SystemTime,
}
