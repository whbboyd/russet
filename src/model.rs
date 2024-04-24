/// Utility types

use serde::Deserialize;
use std::ops::Deref;
use std::time::SystemTime;
use ulid::Ulid;

#[derive(Clone, Deserialize)]
pub struct Timestamp(pub SystemTime);
impl Timestamp {
	pub fn new(time: SystemTime) -> Timestamp {
		Timestamp(time)
	}
}
impl std::fmt::Debug for Timestamp {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", "some timestamp"))
	}
}

#[derive(Clone, Deserialize, Debug)]
pub struct Pagination {
	page_num: usize,
	page_size: usize,
}

#[derive(Clone)]
pub struct FeedId(pub Ulid);
impl Deref for FeedId { type Target = Ulid; fn deref(&self) -> &Self::Target { &self.0 } }
impl std::fmt::Debug for FeedId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("\"{}\"", &self.to_string()))
	}
}
#[derive(Clone, Deserialize)]
pub struct EntryId(pub Ulid);
impl Deref for EntryId{ type Target = Ulid; fn deref(&self) -> &Self::Target { &self.0 } }
impl std::fmt::Debug for EntryId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("\"{}\"", &self.to_string()))
	}
}
#[derive(Clone)]
pub struct UserId(pub Ulid);
impl Deref for UserId{ type Target = Ulid; fn deref(&self) -> &Self::Target { &self.0 } }
impl std::fmt::Debug for UserId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("\"{}\"", &self.to_string()))
	}
}

