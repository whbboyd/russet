use crate::persistence::RussetEntryPersistenceLayer;
use crate::persistence::sql::SqlDatabase;
use crate::persistence::model::{ Entry, EntryId, FeedId, UserId };
use crate::Result;
use std::time::{ Duration, SystemTime };
use reqwest::Url;
use ulid::Ulid;

impl RussetEntryPersistenceLayer for SqlDatabase {

	#[tracing::instrument]
	async fn add_entry(&self, entry: &Entry, feed_id: &FeedId) -> Result<()> {
		let entry_id = entry.id.to_string();
		let feed_id = feed_id.to_string();
		let article_date: i64 = entry.article_date.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().try_into().unwrap();
		let entry_url = entry.url.clone().map(|url| url.to_string());
		sqlx::query!("
				INSERT INTO entries (
					id, feed_id, internal_id, fetch_index, article_date, title, url
				) VALUES ( ?, ?, ?, ?, ?, ?, ? )",
				entry_id,
				feed_id,
				entry.internal_id,
				entry.fetch_index,
				article_date,
				entry.title,
				entry_url,
			)
			.execute(&self.pool)
			.await?;
		Ok(())
	}

	#[tracing::instrument]
	async fn get_entry(&self, id: &EntryId) -> Result<Entry> {
		let entry_id = id.to_string();
		let row = sqlx::query!("
				SELECT
					id, feed_id, internal_id, fetch_index, article_date, title, url
				FROM entries
				WHERE id = ?;",
				entry_id,
			)
			.fetch_one(&self.pool)
			.await?;
		let id = EntryId(Ulid::from_string(&row.id)?);
		let feed_id = FeedId(Ulid::from_string(&row.feed_id)?);
		let article_date = SystemTime::UNIX_EPOCH + Duration::from_millis(row.article_date.try_into().unwrap()); //FIXME
		let url = row.url.map(|url| Url::parse(&url)).transpose()?;
		Ok(Entry {
			id,
			feed_id,
			internal_id: row.internal_id,
			fetch_index: row.fetch_index as u32,
			article_date,
			title: row.title,
			url,
		} )
	}

	#[tracing::instrument]
	async fn get_entries_for_feed(&self, feed_id: &FeedId) -> impl IntoIterator<Item = Result<Entry>> {
		let feed_id = feed_id.to_string();
		// TODO: Maybe do paging later. Or figure out how to stream from sqlx.
		let rows = sqlx::query!("
				SELECT
					id, feed_id, internal_id, fetch_index, article_date, title, url
				FROM entries
				WHERE feed_id = ?
				ORDER BY fetch_index DESC, article_date ASC;",
				feed_id,
			)
			.fetch_all(&self.pool)
			.await;
		let rv: Vec<Result<Entry>> = match rows {
			Ok(rows) => {
				rows.into_iter().map(|row| {
					let id = EntryId(Ulid::from_string(&row.id)?);
					let feed_id = FeedId(Ulid::from_string(&row.feed_id)?);
					let article_date = SystemTime::UNIX_EPOCH + Duration::from_millis(row.article_date.try_into().unwrap()); //FIXME
					let url = row.url.map(|url| Url::parse(&url)).transpose()?;
					Ok(Entry {
						id,
						feed_id,
						internal_id: row.internal_id,
						fetch_index: row.fetch_index as u32,
						article_date,
						title: row.title,
						url,
					} )
				} )
					.collect()
			},
			Err(e) => vec![Err(Box::new(e))],
		};
		rv
	}

	#[tracing::instrument]
	async fn get_and_increment_fetch_index(&self) -> Result<u32> {
		let row = sqlx::query!("
				UPDATE metadata
				SET fetch_index = fetch_index + 1
				RETURNING fetch_index"
			)
			.fetch_one(&self.pool)
			.await?;
		let index = (row.fetch_index - 1).try_into()?;
		Ok(index)
	}

	#[tracing::instrument]
	async fn get_entries_for_user(&self, user_id: &UserId) -> impl IntoIterator<Item = Result<Entry>> {
		let user_id = user_id.to_string();
		// TODO: Maybe do paging later. Or figure out how to stream from sqlx.
		let rows = sqlx::query!(r#"
				SELECT
					e.id AS "id!",
					e.feed_id AS "feed_id!",
					e.internal_id AS "internal_id!",
					e.fetch_index AS "fetch_index!",
					e.article_date AS "article_date!",
					e.title AS "title!",
					e.url
				FROM entries AS e
				JOIN subscriptions AS s
				ON e.feed_id = s.feed_id
				WHERE s.user_id = ?
				ORDER BY fetch_index DESC, article_date ASC;"#,
				user_id,
			)
			.fetch_all(&self.pool)
			.await;
		let rv: Vec<Result<Entry>> = match rows {
			Ok(rows) => {
				rows.into_iter().map(|row| {
					let id = EntryId(Ulid::from_string(&row.id)?);
					let feed_id = FeedId(Ulid::from_string(&row.feed_id)?);
					let article_date = SystemTime::UNIX_EPOCH + Duration::from_millis(row.article_date.try_into().unwrap()); //FIXME
					let url = row.url.map(|url| Url::parse(&url)).transpose()?;
					Ok(Entry {
						id,
						feed_id,
						internal_id: row.internal_id,
						fetch_index: row.fetch_index as u32,
						article_date,
						title: row.title,
						url,
					} )
				} )
					.collect()
			},
			Err(e) => vec![Err(Box::new(e))],
		};
		rv
	}
}
