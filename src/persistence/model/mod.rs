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

#[derive(Debug)]
pub struct UserId(pub Ulid);
impl Deref for UserId{ type Target = Ulid; fn deref(&self) -> &Self::Target { &self.0 } }
#[derive(Debug)]
pub struct User {
	pub id: UserId,
	pub name: String,
	pub password_hash: String,
}

pub struct Session {
	pub token: String,
	
}
