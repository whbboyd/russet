use crate::model::{ EntryId, FeedId };

pub struct Feed {
	pub id: FeedId,
	pub url: String,
	pub title: String,
}

pub struct Entry {
	pub id: EntryId,
	pub feed_id: FeedId,
	pub url: Option<String>,
	pub title: String,
	pub article_date: String,
	pub read: bool,
	pub tombstone: bool,
}
