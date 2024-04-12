use crate::domain::RussetDomainService;
use crate::Result;
use crate::persistence::model::{ Entry, EntryId, Feed, FeedId };
use crate::persistence::{ RussetEntryPersistenceLayer, RussetFeedPersistenceLayer };
use crate::feed::model::Feed as ReaderFeed;
use reqwest::Url;
use std::collections::HashSet;
use ulid::Ulid;

impl <Persistence> RussetDomainService<Persistence>
where Persistence: RussetEntryPersistenceLayer + RussetFeedPersistenceLayer {

	/// Update the stored entries for all feeds known to the persistence layer
	pub async fn update_feeds(&self) -> Result<()> {
		let fetch_index = self.persistence.get_and_increment_fetch_index().await?;
		let feeds = self.persistence
			.get_feeds()
			.await
			.into_iter()
			.filter_map(|feed| feed.ok())
			.collect::<Vec<Feed>>();
		for feed in feeds {
			self.update_feed(&feed, fetch_index).await?;
		}
		Ok(())
	}

	/// Given a URL, ensure the feed is stored in the persistence layer.
	///
	/// If a feed with that URL is already stored, its entries will be updated.
	/// Otherwise, the feed will be downloaded and added to the persistence
	/// layer.
	pub async fn add_feed(&self, url: &Url) -> Result<FeedId> {
		match self.persistence.get_feed_by_url(url).await? {
			Some(feed) => {
				let fetch_index = self.persistence.get_and_increment_fetch_index().await?;
				self.update_feed(&feed, fetch_index).await?;
				Ok(feed.id)
			}
			None => {
				let bytes = reqwest::get(url.clone())
					.await?
					.bytes()
					.await?;
				let reader_feed = self.feed_from_bytes(&bytes).await?;
				let feed = Feed {
					id: FeedId(Ulid::new()),
					title: reader_feed.title.clone(),
					url: url.clone(),
				};
				self.persistence.add_feed(&feed).await?;
				let fetch_index = self.persistence.get_and_increment_fetch_index().await?;
				self.update_with_entries(&feed, &reader_feed, fetch_index).await?;
				Ok(feed.id)
			}
		}
	}

	

	/// Update the persistence layer with [feed] (at fetch [fetch_index])
	async fn update_feed(&self, feed: &Feed, fetch_index: u32) -> Result<()> {
		let bytes = reqwest::get(feed.url.clone())
				.await?
				.bytes()
				.await?;
		// TODO: Store a reader hint with the feed to save redundant parsing effort
		let reader_feed = self.feed_from_bytes(&bytes).await?;
		self.update_with_entries(feed, &reader_feed, fetch_index).await
	}

	/// Given a parsed [reader_feed], update the persistence layer for [feed]
	/// with the entries from it
	async fn update_with_entries(&self, feed: &Feed, reader_feed: &ReaderFeed, fetch_index: u32) -> Result<()> {
		let known_internal_ids = self.persistence
			.get_entries_for_feed(&feed.id)
			.await
			.into_iter()
			.filter_map(|entry| entry.ok().map(|entry| entry.internal_id) )
			.collect::<HashSet<String>>();
		let new_entries = reader_feed.entries.as_slice().into_iter()
			.filter(|entry| !known_internal_ids.contains(&entry.internal_id) )
			.map (|entry| {
				Entry {
					id: EntryId(Ulid::new()),
					feed_id: FeedId(feed.id.0.clone()),
					internal_id: entry.internal_id.clone(),
					fetch_index,
					article_date: entry.article_date,
					title: entry.title.clone(),
					url: entry.url.clone(),
				}
			} )
			.collect::<Vec<Entry>>();
		for e in new_entries.as_slice() {
			self.persistence.add_entry(e, &feed.id).await?;
		}
		Ok(())
	}

	/// Given a 
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

