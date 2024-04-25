pub mod model;
pub mod sql;

use crate::Result;
use crate::model::{ EntryId, FeedId, Pagination, UserId };
use model::{ Entry, Feed, Session, User, UserEntry };
use reqwest::Url;
use std::future::Future;

pub trait RussetPersistenceLayer:
	RussetFeedPersistenceLayer +
	RussetEntryPersistenceLayer +
	RussetUserPersistenceLayer
	{ }

pub trait RussetFeedPersistenceLayer: Send + Sync + std::fmt::Debug + 'static {
	/// Add the given [Feed] to this persistence layer
	fn add_feed(&self, feed: &Feed) -> impl Future<Output = Result<()>> + Send;

	/// Get all the [Feed]s stored by this persistence layer
	async fn get_feeds(&self)
		-> impl IntoIterator<Item = Result<Feed>, IntoIter = impl Iterator<Item = Result<Feed>> + Send>;

	/// Get a specific [Feed] by ID
	async fn get_feed(&self, id: &FeedId) -> Result<Feed>;

	/// Get a specified [Feed] by URL
	fn get_feed_by_url(&self, url: &Url)
		-> impl Future<Output = Result<Option<Feed>>> + Send;
}

pub trait RussetEntryPersistenceLayer: Send + Sync + std::fmt::Debug + 'static {
	/// Add the given [Entry] to this persistence layer
	fn add_entry(&self, entry: &Entry, feed_id: &FeedId)
		-> impl Future<Output = Result<()>> + Send;

	/// Get a specified [Entry] by ID
	fn get_entry(&self, id: &EntryId)
		-> impl Future<Output = Result<Entry>> + Send;

	/// Get all the [Entry]s for the [Feed] with the given ID.
	fn get_entries_for_feed(&self, feed_id: &FeedId)
		-> impl Future<Output = impl IntoIterator<Item = Result<Entry>>> + Send;

	/// Atomically get-and-increment the fetch index.
	fn get_and_increment_fetch_index(&self)
		-> impl Future<Output = Result<u32>> + Send;

	/// get all the entries for all the feeds to which the given user is subscribed.
	fn get_entries_for_user(&self, user_id: &UserId, pagination: &Pagination)
		-> impl Future<Output = impl IntoIterator<Item = Result<(Entry, Option<UserEntry>)>>> + Send;

	fn get_entry_and_set_userentry(
		&self,
		entry_id: &EntryId,
		user_id: &UserId,
		user_entry: &UserEntry,
	) -> impl Future<Output = Result<Entry>> + Send;
}

pub trait RussetUserPersistenceLayer: Send + Sync + std::fmt::Debug + 'static {
	/// Add the given [User] to the persistence layer
	async fn add_user(&self, user: &User) -> Result<()>;

	/// Given a username, look up that user
	fn get_user_by_name(&self, user_name: &str)
		-> impl Future<Output = Result<Option<User>>> + Send;

	/// Add the given [Session] to the persistence layer, logging in a user
	fn add_session(&self, session: &Session)
		-> impl Future<Output = Result<()>> + Send;

	/// Given a session token, look up that user and session
	fn get_user_by_session(&self, session_token: &str)
		-> impl Future<Output = Result<Option<(User, Session)>>> + Send;

	async fn delete_session(&self, session_token: &str) -> Result<()>;

	async fn delete_sessions_for_user(&self, user_id: &UserId) -> Result<u32>;

	fn add_subscription(&self, user_id: &UserId, feed_id: &FeedId)
		-> impl Future<Output = Result<()>> + Send;
}
