use crate::domain::RussetDomainService;
use crate::persistence::model::{ Entry, Feed };
use crate::persistence::RussetEntryPersistenceLayer;
use crate::persistence::RussetFeedPersistenceLayer;
use crate::persistence::sql::SqlDatabase;
use crate::Result;

impl RussetDomainService<SqlDatabase> {
	pub async fn get_entries(&self) -> impl IntoIterator<Item = Result<Entry>> {
		let mut acc = Vec::new();
		for feed in self.persistence.get_feeds().await {
			match feed {
				Ok(Feed { id, .. }) => acc.extend(
					self.persistence
						.get_entries_for_feed(&id)
						.await
				),
				Err(e) => acc.push(Err(e))
			}
		}
		acc
	}
}
