pub mod model;
pub mod sql;

use crate::Result;
use model::{ Entry, EntryId, Feed, FeedId, User };
use reqwest::Url;

pub trait RussetPersistenceLayer:
	RussetFeedPersistenceLayer +
	RussetEntryPersistenceLayer +
	RussetUserPersistenceLayer
	{ }

pub trait RussetFeedPersistenceLayer {
	/// Add the given [Feed] to this persistence layer
	fn add_feed(&mut self, feed: &Feed) -> Result<()>;

	/// Get all the [Feed]s stored by this persistence layer
	fn get_feeds(&self) -> impl IntoIterator<Item = Result<Feed>>;

	/// Get a specific [Feed] by ID
	fn get_feed(&self, id: &FeedId) -> Result<Feed>;

	/// Get a specified [Feed] by URL
	fn get_feed_by_url(&self, url: &Url) -> Result<Option<Feed>>;
}

pub trait RussetEntryPersistenceLayer {
	/// Add the given [Entry] to this persistence layer
	fn add_entry(&mut self, entry: &Entry, feed_id: &FeedId) -> Result<()>;

	/// Get a specified [Entry] by ID
	fn get_entry(&self, id: &EntryId) -> Result<Entry>;

	/// Get all the [Entry]s for the [Feed] with the given ID
	fn get_entries_for_feed(&self, feed_id: &FeedId) -> impl IntoIterator<Item = Result<Entry>>;

	/// Atomically get-and-increment the fetch index.
	fn get_and_increment_fetch_index(&mut self) -> Result<u32>;
}

pub trait RussetUserPersistenceLayer {
	/// Add the given [User] to the persistence layer
	fn add_user(&mut self, user: &User) -> Result<()>;

	/// Given a username, look up that user
	fn get_user_by_name(&self, user_name: &str) -> Result<Option<User>>;
}
