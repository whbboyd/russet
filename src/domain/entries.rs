use chrono::{ DateTime, TimeDelta, Utc };
use chrono_tz::Tz;
use crate::domain::model::Entry;
use crate::domain::RussetDomainService;
use crate::persistence::model::{ Entry as PersistenceEntry, UserId };
use crate::persistence::RussetEntryPersistenceLayer;
use crate::Result;

impl <Persistence> RussetDomainService<Persistence>
where Persistence: RussetEntryPersistenceLayer {
	pub async fn get_subscribed_entries(&self, user_id: &UserId) -> impl IntoIterator<Item = Result<Entry>> {
		self.persistence
			.get_entries_for_user(user_id)
			.await
			.into_iter()
			.map(|result| result.map(|entry| convert_entry(entry, /*FIXME*/Tz::UTC)))
			.collect::<Vec<Result<Entry>>>()
	}
}

fn convert_entry(entry: PersistenceEntry, tz: Tz) -> Entry {
	// Article date: full ISO8601 out to -2 days, then ISO8601 date
	let article_date_utc: DateTime<Utc> = entry.article_date.into();
	let article_date = article_date_utc.with_timezone(&tz);
	let article_date_str = if (Utc::now() - article_date_utc) < TimeDelta::days(2) {
		article_date.to_rfc3339()
	} else {
		article_date.date_naive().format("%Y-%m-%d").to_string()
	};
	Entry {
		id: entry.id.to_string(),
		url: entry.url.map(|url| url.to_string()),
		title: entry.title,
		article_date: article_date_str,
	}
}
