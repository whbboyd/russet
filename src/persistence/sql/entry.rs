use crate::model::{ EntryId, FeedId, Pagination, UserId };
use crate::persistence::RussetEntryPersistenceLayer;
use crate::persistence::sql::SqlDatabase;
use crate::persistence::model::{ Entry, UserEntry };
use crate::Result;
use reqwest::Url;
use ulid::Ulid;

impl RussetEntryPersistenceLayer for SqlDatabase {

	#[tracing::instrument]
	async fn add_entry(&self, entry: &Entry, feed_id: &FeedId) -> Result<()> {
		let entry_id = entry.id.to_string();
		let feed_id = feed_id.to_string();
		let article_date: i64 = entry.article_date.clone().try_into()?;
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
		let url = row.url.map(|url| Url::parse(&url)).transpose()?;
		Ok(Entry {
			id,
			feed_id,
			internal_id: row.internal_id,
			fetch_index: row.fetch_index as u32,
			article_date: row.article_date.into(),
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
				ORDER BY fetch_index DESC, article_date DESC;",
				feed_id,
			)
			.fetch_all(&self.pool)
			.await;
		let rv: Vec<Result<Entry>> = match rows {
			Ok(rows) => {
				rows.into_iter().map(|row| {
					let id = EntryId(Ulid::from_string(&row.id)?);
					let feed_id = FeedId(Ulid::from_string(&row.feed_id)?);
					let url = row.url.map(|url| Url::parse(&url)).transpose()?;
					Ok(Entry {
						id,
						feed_id,
						internal_id: row.internal_id,
						fetch_index: row.fetch_index as u32,
						article_date: row.article_date.into(),
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
	async fn get_entries_for_user(
		&self,
		user_id: &UserId,
		pagination: &Pagination,
	) -> Vec<Result<(Entry, Option<UserEntry>)>> {
		self.get_userentries(user_id, None, None, pagination).await
	}

	#[tracing::instrument]
	async fn get_entries_for_user_feed(
		&self,
		user_id: &UserId,
		feed_id: &FeedId,
		pagination: &Pagination,
	) -> impl IntoIterator<Item = Result<(Entry, Option<UserEntry>)>> {
		self.get_userentries(user_id, Some(feed_id), None, pagination).await
	}

	#[tracing::instrument]
	async fn get_entry_and_set_userentry(
		&self,
		entry_id: &EntryId,
		user_id: &UserId,
		user_entry: &UserEntry
	) -> Result<Entry> {
		let entry_id = entry_id.to_string();
		let user_id = user_id.to_string();
		let read: Option<i64> = user_entry.read
			.clone()
			.and_then(|timestamp| timestamp.try_into().ok());
		let tombstone: Option<i64> = user_entry.tombstone
			.clone()
			.and_then(|timestamp| timestamp.try_into().ok());
		let mut tx = self.pool.begin().await?;
		// Query the entry first to make sure it actually exists
		let row = sqlx::query!("
				SELECT
					id, feed_id, internal_id, fetch_index, article_date, title, url
				FROM entries
				WHERE id = ?;",
				entry_id,
			)
			.fetch_one(&mut *tx)
			.await?;
		sqlx::query!("
				INSERT INTO user_entry_settings (
					user_id, entry_id, read, tombstone
				) VALUES ( ?, ?, ?, ?)
				ON CONFLICT (user_id, entry_id)
				DO UPDATE SET
					read = excluded.read,
					tombstone = excluded.tombstone;",
				user_id,
				entry_id,
				read,
				tombstone,
			)
			.execute(&mut *tx)
			.await?;
		tx.commit().await?;
		let id = EntryId(Ulid::from_string(&row.id)?);
		let feed_id = FeedId(Ulid::from_string(&row.feed_id)?);
		let url = row.url.map(|url| Url::parse(&url)).transpose()?;
		Ok(Entry {
			id,
			feed_id,
			internal_id: row.internal_id,
			fetch_index: row.fetch_index as u32,
			article_date: row.article_date.into(),
			title: row.title,
			url,
		} )
	}
}

impl SqlDatabase {
	/// Helper for entry/user_entry fetching.
	async fn get_userentries(
		&self,
		user_id: &UserId,
		feed_id: Option<&FeedId>,
		entry_id: Option<&EntryId>,
		pagination: &Pagination,
	) -> Vec<Result<(Entry, Option<UserEntry>)>> {
		let user_id_str = user_id.to_string();
		let no_feed = feed_id.is_none();
		let feed_id_str = feed_id.map(|id| id.to_string());
		let no_entry = entry_id.is_none();
		let entry_id_str = entry_id.map(|id| id.to_string());
		let page_size: i64 = match pagination.page_size.try_into() {
			Ok(i) => i,
			Err(e) => return vec![Err(e.into())]
		};
		let page_offset: i64 = match (pagination.page_num * pagination.page_size).try_into() {
			Ok(i) => i,
			Err(e) => return vec![Err(e.into())]
		};
		// TODO: Maybe do paging later. Or figure out how to stream from sqlx.

		// This query is this way because in order to pass it to query!, it must
		// be a &'static str, which means no dynamically-added query clauses.
		// The (? OR id = ?) clauses allow us to skip these checks if we weren't
		// provied an ID to check against.
		let rows = sqlx::query!(r#"
				SELECT
					e.id AS "id!",
					e.feed_id AS "feed_id!",
					e.internal_id AS "internal_id!",
					e.fetch_index AS "fetch_index!",
					e.article_date AS "article_date!",
					e.title AS "title!",
					e.url,
					u.user_id AS "user_entry_user_id",
					u.read,
					u.tombstone
				FROM entries AS e
				INNER JOIN subscriptions AS s
					ON e.feed_id = s.feed_id
				LEFT OUTER JOIN user_entry_settings AS u
					ON s.user_id = u.user_id AND e.id = u.entry_id
				WHERE s.user_id = ?
					AND (? OR s.feed_id = ?)
					AND (? OR e.id = ?)
				ORDER BY fetch_index DESC, article_date DESC
				LIMIT ?
				OFFSET ?;"#,
				user_id_str,
				no_feed,
				feed_id_str,
				no_entry,
				entry_id_str,
				page_size,
				page_offset,
			)
			.fetch_all(&self.pool)
			.await;
		let rv: Vec<Result<(Entry, Option<UserEntry>)>> = match rows {
			Ok(rows) => {
				rows.into_iter().map(|row| {
					let id = EntryId(Ulid::from_string(&row.id)?);
					let feed_id = FeedId(Ulid::from_string(&row.feed_id)?);
					let url = row.url.map(|url| Url::parse(&url)).transpose()?;
					let entry = Entry {
						id,
						feed_id,
						internal_id: row.internal_id,
						fetch_index: row.fetch_index as u32,
						article_date: row.article_date.into(),
						title: row.title,
						url,
					};
					let user_entry = if row.user_entry_user_id.is_some() {
						Some(UserEntry {
							read: row.read.map(|read| read.into()),
							tombstone: row.tombstone.map(|tombstone| tombstone.into()),
						} )
					} else {
						None
					};
					Ok( (
						entry,
						user_entry,
					) )
				} )
					.collect()
			},
			Err(e) => vec![Err(Box::new(e))],
		};
		rv
	}
}
