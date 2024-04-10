pub mod model;
pub mod sql;

use crate::Result;
use model::{ Entry, EntryId, Feed, FeedId, Session, User, UserId };
use reqwest::Url;
use std::future::Future;

pub trait RussetPersistenceLayer:
	RussetFeedPersistenceLayer +
	RussetEntryPersistenceLayer +
	RussetUserPersistenceLayer
	{ }

pub trait RussetFeedPersistenceLayer {
	/// Add the given [Feed] to this persistence layer
	async fn add_feed(&self, feed: &Feed) -> Result<()>;

	/// Get all the [Feed]s stored by this persistence layer
	async fn get_feeds(&self) -> impl IntoIterator<Item = Result<Feed>, IntoIter = impl Iterator<Item = Result<Feed>> + Send>;

	/// Get a specific [Feed] by ID
	async fn get_feed(&self, id: &FeedId) -> Result<Feed>;

	/// Get a specified [Feed] by URL
	async fn get_feed_by_url(&self, url: &Url) -> Result<Option<Feed>>;
}

pub trait RussetEntryPersistenceLayer {
	/// Add the given [Entry] to this persistence layer
	async fn add_entry(&self, entry: &Entry, feed_id: &FeedId) -> Result<()>;

	/// Get a specified [Entry] by ID
	async fn get_entry(&self, id: &EntryId) -> Result<Entry>;

	/// Get all the [Entry]s for the [Feed] with the given ID
	async fn get_entries_for_feed(&self, feed_id: &FeedId) -> impl IntoIterator<Item = Result<Entry>>;

	/// Atomically get-and-increment the fetch index.
	async fn get_and_increment_fetch_index(&self) -> Result<u32>;
}

pub trait RussetUserPersistenceLayer {
	/// Add the given [User] to the persistence layer
	async fn add_user(&self, user: &User) -> Result<()>;

	/// Given a username, look up that user
	async fn get_user_by_name(&self, user_name: &str) -> Result<Option<User>>;

	/// Add the given [Session] to the persistence layer, logging in a user
	async fn add_session(&self, session: &Session) -> Result<()>;

	/// Given a session token, look up that user and session
//	async fn get_user_by_session(&self, session_token: &str) -> Result<Option<(User, Session)>>;
	fn get_user_by_session(&self, session_token: &str) -> impl Future<Output = Result<Option<(User, Session)>>> + Send;

	async fn delete_session(&self, session_token: &str) -> Result<()>;

	async fn delete_sessions_for_user(&self, user_id: &UserId) -> Result<u32>;

	async fn add_subscription(&self, user_id: &UserId, feed_id: &FeedId) -> Result<()>;
}
