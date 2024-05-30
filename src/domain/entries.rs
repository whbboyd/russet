use chrono::{ DateTime, TimeDelta, Utc };
use chrono_tz::Tz;
use crate::domain::model::Entry;
use crate::domain::RussetDomainService;
use crate::model::{ EntryId, FeedId, Pagination, Timestamp, UserId };
use crate::persistence::model::{ Entry as PersistenceEntry, UserEntry };
use crate::persistence::RussetEntryPersistenceLayer;
use crate::Result;
use std::time::SystemTime;

impl <Persistence> RussetDomainService<Persistence>
where Persistence: RussetEntryPersistenceLayer {
	pub async fn get_subscribed_entries(
		&self,
		user_id: &UserId,
		pagination: &Pagination
	) -> impl IntoIterator<Item = Result<Entry>> {
		self.persistence
			.get_entries_for_user(user_id, pagination)
			.await
			.into_iter()
			.map(|result| result.map(|(entry, user_entry)| convert_entry(entry, user_entry, /*FIXME*/Tz::UTC)))
			.filter(|entry| entry.as_ref().map_or_else(|_| true, |entry| !entry.tombstone))
			.collect::<Vec<Result<Entry>>>()
	}

	pub async fn get_entry(&self, entry_id: &EntryId, user_id: &UserId) -> Result<Entry> {
		let user_entry = UserEntry {
			read: Some(Timestamp::new(SystemTime::now())),
			tombstone: None,
		};
		Ok(self.persistence
			.get_entry_and_set_userentry(entry_id, user_id, &user_entry)
			.await
			.map(|entry| convert_entry(entry, Some(user_entry), Tz::UTC))?
		)
	}

	pub async fn get_feed_entries(
		&self,
		user_id: &UserId,
		feed_id: &FeedId,
		pagination: &Pagination,
	) -> impl IntoIterator<Item = Result<Entry>> {
		self.persistence
			.get_entries_for_user_feed(user_id, feed_id, pagination)
			.await
			.into_iter()
			.map(|result| result.map(|(entry, user_entry)| convert_entry(entry, user_entry, /*FIXME*/Tz::UTC)))
			.filter(|entry| entry.as_ref().map_or_else(|_| true, |entry| !entry.tombstone))
			.collect::<Vec<Result<Entry>>>()
	}

	pub async fn set_userentries(
		&self,
		entry_ids: &Vec<EntryId>,
		user_id: &UserId,
		user_entry: &UserEntry,
	) -> Result<()> {
		for entry_id in entry_ids {
			let _ = self.persistence
				.get_entry_and_set_userentry(entry_id, user_id, user_entry)
				.await?;
		}
		Ok(())
	}

}

fn convert_entry(entry: PersistenceEntry, user_entry: Option<UserEntry>, tz: Tz) -> Entry {
	// Article date: full ISO8601 out to -2 days, then ISO8601 date
	let article_date_utc: DateTime<Utc> = entry.article_date.0.into();
	let article_date = article_date_utc.with_timezone(&tz);
	let article_date_str = if (Utc::now() - article_date_utc) < TimeDelta::days(2) {
		article_date.to_rfc3339()
	} else {
		article_date.date_naive().format("%Y-%m-%d").to_string()
	};
	Entry {
		id: entry.id,
		feed_id: entry.feed_id,
		url: entry.url.map(|url| url.to_string()),
		title: entry.title,
		article_date: article_date_str,
		read: user_entry.as_ref().and_then(|user_entry| user_entry.read.as_ref()).is_some(),
		tombstone: user_entry.as_ref().and_then(|user_entry| user_entry.tombstone.as_ref()).is_some(),
	}
}
