use crate::model::{ EntryId, FeedId };

pub struct Feed {
	pub id: FeedId,
	pub url: String,
	pub title: String,
}
impl From<crate::persistence::model::Feed> for Feed {
	fn from(value: crate::persistence::model::Feed) -> Self {
		Feed {
			id: value.id,
			url: value.url.to_string(),
			title: value.title,
		}
	}
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
