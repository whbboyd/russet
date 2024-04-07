use reqwest::Url;
use std::ops::Deref;
use std::time::SystemTime;
use ulid::Ulid;

#[derive(Debug)]
pub struct FeedId(pub Ulid);
impl Deref for FeedId { type Target = Ulid; fn deref(&self) -> &Self::Target { &self.0 } }
/// Metadata for a feed, e.g. title and feed URL
#[derive(Debug)]
pub struct Feed {
	pub id: FeedId,
	pub title: String,
	pub url: Url,
}

#[derive(Debug)]
pub struct EntryId(pub Ulid);
impl Deref for EntryId{ type Target = Ulid; fn deref(&self) -> &Self::Target { &self.0 } }
/// Individual entry from a given feed
#[derive(Debug)]
pub struct Entry {
	pub id: EntryId,
	pub feed_id: FeedId,
	pub internal_id: String,
	pub fetch_index: u32,
	pub article_date: SystemTime,
	pub title: String,
	pub url: Option<Url>,
}

#[derive(Clone, Debug)]
pub struct UserId(pub Ulid);
impl Deref for UserId{ type Target = Ulid; fn deref(&self) -> &Self::Target { &self.0 } }
pub struct User {
	pub id: UserId,
	pub name: String,
	pub password_hash: String,
}
impl std::fmt::Debug for User {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("User")
			.field("id", &self.id.to_string())
			.field("name", &self.name)
			.field("password_hash", &"<redacted>")
			.finish()
	}
}

pub struct Session {
	pub token: String,
	pub user_id: UserId,
	pub expiration: SystemTime,
}
impl std::fmt::Debug for Session {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Session")
			.field("token", &format!("{}<redacted>", &self.token[..4]))
			.field("user_id", &self.user_id.to_string())
			.field("expiration", &self.expiration)
			.finish()
	}
}
