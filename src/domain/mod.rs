
use crate::async_util::AsyncUtil;
use crate::{ Err, Result };
use crate::feed::RussetFeedReader;
use crate::persistence::model::{ Entry, Feed };
use crate::persistence::RussetPersistanceLayer;
use reqwest::Url;
use std::sync::Arc;
use ulid::Ulid;

pub struct RussetDomainService<Persistence>
where Persistence: RussetPersistanceLayer {
	persistence: Persistence,
	readers: Vec<Box<dyn RussetFeedReader>>,
	async_util: Arc<AsyncUtil>,
}
impl <Persistence> RussetDomainService<Persistence>
where Persistence: RussetPersistanceLayer {
	pub fn new(
		persistence: Persistence,
		readers: Vec<Box<dyn RussetFeedReader>>,
		async_util: Arc<AsyncUtil>,
	) -> RussetDomainService<Persistence> {
		RussetDomainService {
			persistence,
			readers,
			async_util,
		}
	}

	pub fn update_feeds(&mut self) -> Result<()> {
		let fetch_index = self.persistence.get_and_increment_fetch_index()?;
		let feeds = self.persistence.get_feeds().into_iter().collect::<Vec<Result<Feed>>>();
		for feed in feeds {
			if let Ok(feed) = feed {
				self.update_feed(&feed, fetch_index)?;
			}
		}
		Ok(())
	}

	pub fn add_feed(&mut self, url: &Url) -> Result<()> {
		match self.persistence.get_feed_by_url(url) {
			Ok(feed) => {
				let fetch_index = self.persistence.get_and_increment_fetch_index()?;
				self.update_feed(&feed, fetch_index)
			}
			Err(_) => {
				todo!()
			}
		}
	}

	fn update_feed(&mut self, feed: &Feed, fetch_index: u32) -> Result<()> {
		// TODO: Store a reader hint with the feed to save redundant parsing effort
		let bytes = self.async_util.run_blocking(|| async {
			reqwest::get(feed.url.clone())
				.await?
				.bytes()
				.await
		} )?;
		let reader_feed = self.readers.as_slice().into_iter()
			.find_map(|reader| reader.read_feed(&bytes).ok())
			.ok_or::<Err>("Unable to load feed".into())?;
		let storage_entries = self.persistence
			.get_entries_for_feed(&feed.id)
			.into_iter()
			.collect::<Vec<Result<Entry>>>();
		let new_entries = reader_feed.entries.into_iter()
			.filter(|entry| {
				for s in storage_entries.as_slice() {
					if s.as_ref().map(|e| e.internal_id == entry.internal_id).unwrap_or(false) {
						return false
					}
				};
				true
			} )
			.map (|entry| {
				Entry {
					id: Ulid::new(),
					internal_id: entry.internal_id,
					fetch_index,
					article_date: entry.article_date,
					title: entry.title,
					url: entry.url,
				}
			} )
			.collect::<Vec<Entry>>();
		for e in new_entries.as_slice() {
			self.persistence.add_entry(e, &feed.id)?;
		}
		Ok(())
	}
}

