mod update;

use crate::domain::model::Feed;
use crate::domain::RussetDomainService;
use crate::Result;
use crate::model::{ EntryId, FeedId, UserId, Timestamp };
use crate::persistence::model::{ Entry, Feed as PersistenceFeed, FeedCheck, WriteFeedCheck };
use crate::persistence::{ RussetEntryPersistenceLayer, RussetFeedPersistenceLayer };
use crate::feed::model::Feed as ReaderFeed;
use reqwest::Url;
use std::cmp::Ordering::{ Equal, Greater, Less };
use std::collections::HashSet;
use tracing::warn;
use ulid::Ulid;

impl <Persistence> RussetDomainService<Persistence>
where Persistence: RussetEntryPersistenceLayer + RussetFeedPersistenceLayer {

	/// Update the stored entries for the given feed.
	///
	/// Returns the [FeedCheck] generated from this update.
	pub async fn update_feed(&self, feed_id: &FeedId) -> Result<FeedCheck> {
		let now = Timestamp::now();
		let feed = self.persistence.get_feed(feed_id).await?;
		let last_check = self.persistence.get_last_feed_check(feed_id).await?;
		let check_time = last_check.map_or(now, |check| check.next_check_time);
		match now.cmp(&check_time) {
			Less => {
				warn!("Check was scheduled with future check time ({check_time:?}; now: {now:?})")
			}
			Equal => (), // Normal: check time is current time
			Greater => (), // We missed the check. This is normal, but if we missed the check by a lot, we may want to know.
		}

		// Fetch the feed data. We do this now (before recording the check)
		// because some of its details will need to feed back into the check.
		// TODO: include the etag and handle errors (flag on the check) here
		let reader_feed = self.fetch(&feed.url).await?;

		// Now, generate the check. We need this to store the entries, because
		// they must be tagged with the check that generated them.
		let check = self
			.build_check_and_update(check_time, feed_id, &reader_feed)
			.await?;

		Ok(check)
	}

	/// Get all feeds known to the persistence layer
	pub async fn get_feeds(&self) -> impl IntoIterator<Item = Result<Feed>> + '_ {
		self.persistence
			.get_feeds()
			.await
			.into_iter()
			.map(|feed| {
				feed.map(|feed| { feed.into() } )
			} )
	}

	/// Given a URL, ensure the feed is stored in the persistence layer.
	///
	/// If a feed with that URL is already stored, no action is taken.
	/// Otherwise, the feed will be downloaded and added to the persistence
	/// layer.
	pub async fn add_feed(&self, url: &Url) -> Result<FeedId> {
		match self.persistence.get_feed_by_url(url).await? {
			Some(feed) => {
				// TODO: Contemplate rescheduling the next check in this case.
				Ok(feed.id)
			}
			None => {
				let reader_feed = self.fetch(url).await?;
				let feed = PersistenceFeed {
					id: FeedId(Ulid::new()),
					title: reader_feed.title.clone(),
					url: url.clone(),
				};
				self.persistence.add_feed(&feed).await?;
				// TODO: Add the feed to scheduling.
				self.build_check_and_update(
						Timestamp::now(),
						&feed.id,
						&reader_feed
					).await?;

				Ok(feed.id)
			}
		}
	}

	pub async fn feeds_for_user(&self, user_id: &UserId) -> Vec<Result<Feed>> {
		self.persistence
			.get_subscribed_feeds(user_id)
			.await
			.into_iter()
			.map(|feed| {
				feed.map(|feed| { feed.into() } )
			} )
			.collect()
	}

	pub async fn get_feed(&self, feed_id: &FeedId) -> Result<Feed> {
		self.persistence
			.get_feed(feed_id)
			.await
			.map(|feed| { feed.into() } )
	}

	/// Fetch feed data from the remote system
	async fn fetch(&self, url: &Url) -> Result<ReaderFeed> {
		let bytes = reqwest::get(url.clone())
				.await?
				.bytes()
				.await?;
		// TODO: Store a reader hint with the feed to save redundant parsing effort
		let reader_feed = self.feed_from_bytes(&bytes).await?;
		Ok(reader_feed)
	}

	/// Given a parsed `reader_feed`, generate and persist a [FeedCheck] for it
	/// and update the persistence layer with its entries
	async fn build_check_and_update(
		&self,
		check_time: Timestamp,
		feed_id: &FeedId,
		reader_feed: &ReaderFeed,
	) -> Result<FeedCheck> {
		// Generate the check. We need this to store the entries, because
		// they must be tagged with the check that generated them.
		// TODO: Whole lotta logic goes here:
		let next_check_time = check_time + self.default_feed_check_interval;
		let check = self.persistence.add_feed_check(WriteFeedCheck {
			feed_id: feed_id.clone(),
			check_time,
			next_check_time,
			etag: None,
		} ).await?;

		// Finally, store the entries, tagged with the check.
		self.update_with_entries(feed_id, &reader_feed, check.id).await?;

		Ok(check)
	}

	/// Given a parsed `reader_feed`, update the persistence layer for the given
	/// feed with the entries from it
	async fn update_with_entries(&self, feed_id: &FeedId, reader_feed: &ReaderFeed, check_id: u64) -> Result<()> {
		let known_internal_ids = self.persistence
			.get_entries_for_feed(&feed_id)
			.await
			.into_iter()
			.filter_map(|entry| entry.ok().map(|entry| entry.internal_id) )
			.collect::<HashSet<String>>();
		let new_entries = reader_feed.entries.as_slice().into_iter()
			.filter(|entry| !known_internal_ids.contains(&entry.internal_id) )
			.map (|entry| {
				Entry {
					id: EntryId(Ulid::new()),
					feed_id: feed_id.clone(),
					internal_id: entry.internal_id.clone(),
					check_id,
					article_date: entry.article_date.clone(),
					title: entry.title.clone(),
					url: entry.url.clone(),
				}
			} )
			.collect::<Vec<Entry>>();
		for e in new_entries.as_slice() {
			self.persistence.add_entry(e, &feed_id).await?;
		}
		Ok(())
	}

	/// Given a serialized feed, attempt to deserialize it using all the known
	/// `readers`.
	///
	/// TODO: This always attempts all `readers`, but a given URL will virtually
	/// never change format. We should store a format hint with the feed and
	/// optimistically try that format first, before falling back to others.
	async fn feed_from_bytes(&self, bytes: &[u8]) -> Result<ReaderFeed> {
		let mut acc = Vec::new();
		for reader in self.readers.as_slice() {
			acc.push(async {
				reader.read_feed(&bytes)
			} );
		}
		for future in acc {
			if let Ok(feed) = future.await {
				return Ok(feed)
			}
		}
		Err("Unable to load feed".into())
	}
}

