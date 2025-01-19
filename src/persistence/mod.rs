pub mod model;
pub mod sql;

use crate::Result;
use crate::model::{ EntryId, FeedId, Pagination, Timestamp, UserId };
use model::{ Entry, Feed, FeedCheck, Session, User, UserEntry, WriteFeedCheck };
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
	fn get_feeds(&self)
		-> impl Future<Output = impl IntoIterator<Item = Result<Feed>> + Send> + Send;

	/// Get a specific [Feed] by ID
	fn get_feed(&self, id: &FeedId) -> impl Future<Output = Result<Feed>> + Send;

	/// Get a specific [Feed] by URL
	fn get_feed_by_url(&self, url: &Url)
		-> impl Future<Output = Result<Option<Feed>>> + Send;

	/// Get all the [Feed]s the given user is subscribed to
	fn get_subscribed_feeds(&self, user_id: &UserId)
		-> impl Future<Output = impl IntoIterator<Item = Result<Feed>>> + Send;

	/// Add the given [WriteFeedCheck] to the persistence layer. The persistence
	/// layer will generate the `id`.
	fn add_feed_check(&self, feed_check: WriteFeedCheck)
		-> impl Future<Output = Result<FeedCheck>> + Send;

	/// Get all feed checks for the given feed, in reverse chronological order
	/// (newest to oldest).
	fn get_feed_checks(&self, feed_id: &FeedId, pagination: &Pagination)
		-> impl Future<Output = impl IntoIterator<Item = Result<FeedCheck>>> + Send;

	/// Get the latest feed check for the given feed.
	///
	/// The default implementation calls [get_feed_checks] with a [Pagination]
	/// with `page_size` of 1.
	fn get_last_feed_check(&self, feed_id: &FeedId)
		-> impl Future<Output = Result<Option<FeedCheck>>> + Send
	{ async {
		self.get_feed_checks(feed_id, &Pagination { page_num: 0, page_size: 1 })
			.await
			.into_iter()
			.next()
			.transpose()
	} }
}

pub trait RussetEntryPersistenceLayer: Send + Sync + std::fmt::Debug + 'static {
	/// Add the given [Entry] to this persistence layer
	fn add_entry(&self, entry: &Entry, feed_id: &FeedId)
		-> impl Future<Output = Result<()>> + Send;

	/// Get a specified [Entry] by ID
	#[allow(dead_code)]
	fn get_entry(&self, id: &EntryId)
		-> impl Future<Output = Result<Entry>> + Send;

	/// Get all the [Entry]s for the [Feed] with the given ID.
	fn get_entries_for_feed(&self, feed_id: &FeedId)
		-> impl Future<Output = impl IntoIterator<Item = Result<Entry>>> + Send;

	/// Get entries for all the feeds to which the given user is subscribed.
	fn get_entries_for_user(&self, user_id: &UserId, pagination: &Pagination)
		-> impl Future<Output = impl IntoIterator<Item = Result<(Entry, Option<UserEntry>)>>> + Send;

	/// Get entries for the given feed to which the given user is subscribed
	fn get_entries_for_user_feed(&self, user_id: &UserId, feed_id: &FeedId, pagination: &Pagination)
		-> impl Future<Output = impl IntoIterator<Item = Result<(Entry, Option<UserEntry>)>>> + Send;

	/// Atomically get an entry and set the userentry for the given entry and user.
	fn get_entry_and_set_userentry(
		&self,
		entry_id: &EntryId,
		user_id: &UserId,
		user_entry: &UserEntry,
	) -> impl Future<Output = Result<Entry>> + Send;
}

pub trait RussetUserPersistenceLayer: Send + Sync + std::fmt::Debug + 'static {

	/// Get the [User] with the given [UserId]
	fn get_user(&self, user_id: &UserId) -> impl Future<Output = Result<User>> + Send;

	/// Add the given [User] to the persistence layer
	async fn add_user(&self, user: &User) -> Result<()>;

	/// Update the user with the given [User] in the persistence layer
	async fn update_user(&self, user: &User) -> Result<()>;

	/// Delete the given [User] from the persistence layer
	async fn delete_user(&self, user_id: &UserId) -> Result<()>;

	/// Given a username, look up that user
	fn get_user_by_name(&self, user_name: &str)
		-> impl Future<Output = Result<Option<User>>> + Send;

	/// Add the given [Session] to the persistence layer, logging in a user
	fn add_session(&self, session: &Session)
		-> impl Future<Output = Result<()>> + Send;

	/// Given a session token, look up that user and session
	fn get_user_by_session(&self, session_token: &str)
		-> impl Future<Output = Result<Option<(User, Session)>>> + Send;

	fn delete_session(&self, session_token: &str)
		-> impl Future<Output = Result<()>> + Send;
	
	fn delete_expired_sessions(&self, expiry: &Timestamp)
		-> impl Future<Output = Result<()>> + Send;

	async fn delete_sessions_for_user(&self, user_id: &UserId) -> Result<u32>;

	fn add_subscription(&self, user_id: &UserId, feed_id: &FeedId)
		-> impl Future<Output = Result<()>> + Send;

	fn remove_subscription(&self, user_id: &UserId, feed_id: &FeedId)
		-> impl Future<Output = Result<()>> + Send;
}
