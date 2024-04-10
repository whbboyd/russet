use crate::domain::RussetDomainService;
use crate::persistence::model::{ Entry, Feed };
use crate::persistence::RussetEntryPersistenceLayer;
use crate::persistence::RussetFeedPersistenceLayer;
use crate::persistence::sql::SqlDatabase;
use crate::Result;

impl RussetDomainService<SqlDatabase> {
	pub async fn get_entries(&self) -> Vec<Result<Entry>> {
		// FIXME Everything about this is horrifying.
		let mut acc = Vec::new();
		let feeds = self.persistence.get_feeds().await.into_iter().filter_map(|feed| feed.ok()).collect::<Vec<Feed>>();
		for feed in feeds {
			let entries = self.persistence
				.get_entries_for_feed(&feed.id)
				.await
				.into_iter()
				.filter_map(|entry| entry.ok());
			acc.extend(entries);
		}
		acc.into_iter().map(|entry| Ok(entry)).collect()
	}
}
