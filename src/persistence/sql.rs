use crate::async_util::AsyncUtil;
use crate::persistence::model::{ Entry, EntryId, Feed, FeedId, User, UserId };
use crate::persistence::RussetPersistanceLayer;
use crate::Result;
use reqwest::Url;
use sqlx::{ Pool, Sqlite };
use std::error::Error;
use std::path::Path;
use std::sync::Arc;
use std::time::{ Duration, SystemTime };
use ulid::Ulid;

#[derive(Debug)]
pub struct SqlDatabase {
	pool: Pool<Sqlite>,
	async_util: Arc<AsyncUtil>,
}
impl SqlDatabase {
	pub fn new(db_path: &Path, async_util: Arc<AsyncUtil>) -> Result<SqlDatabase> {
		let path = db_path
			.to_str()
			.ok_or::<Box<dyn Error>>("db_path is not valid UTF-8".into())?;
		let pool = async_util.run_blocking(|| async {
			Pool::<Sqlite>::connect(path).await
		} )?;
		async_util.run_blocking(|| async {
			sqlx::migrate!("db/migrations/").run(&pool).await
		} )?;
		Ok(SqlDatabase { pool, async_util })
	}
}

impl RussetPersistanceLayer for SqlDatabase {

	fn add_feed(&mut self, feed: &Feed) -> Result<()> {
		let feed_id = feed.id.to_string();
		let feed_url = feed.url.to_string();
		self.async_util.run_blocking(|| async {
			sqlx::query!("
					INSERT INTO feeds (
						id, url, title
					) VALUES ( ?, ?, ? )",
					feed_id,
					feed_url,
					feed.title,
				)
				.execute(&self.pool)
				.await
		} )?;
		Ok(())
	}

	fn get_feeds(&self) -> impl IntoIterator<Item = Result<Feed>> {
		// TODO: Maybe do paging later. Or figure out how to stream from sqlx.
		let rows = self.async_util.run_blocking(|| async {
			sqlx::query!("
					SELECT
						id, url, title
					FROM feeds;"
				)
				.fetch_all(&self.pool)
				.await
		} );
		let rv: Vec<Result<Feed>> = match rows {
			Ok(rows) => {
				rows.into_iter().map(|row| {
					let id = FeedId(Ulid::from_string(&row.id)?);
					let url = Url::parse(&row.url)?;
					Ok(Feed {
						id,
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

	fn get_feed(&self, id: &FeedId) -> Result<Feed> {
		let feed_id = id.to_string();
		let row = self.async_util.run_blocking(|| async {
			sqlx::query!("
					SELECT
						url, title
					FROM feeds
					WHERE id = ?;",
					feed_id,
				)
				.fetch_one(&self.pool)
				.await
		} )?;
		let id = FeedId(id.0.clone());
		let url = Url::parse(&row.url)?;
		let title = row.title;
		Ok(Feed { id, url, title })
	}

	fn get_feed_by_url(&self, url: &Url) -> Result<Option<Feed>> {
		let feed_url = url.to_string();
		let row_result = self.async_util.run_blocking(|| async {
			sqlx::query!("
					SELECT
						id, url, title
					FROM feeds
					WHERE url = ?;",
					feed_url)
				.fetch_one(&self.pool)
				.await
		} );
		match row_result {
			Ok(row) => {
				let id = FeedId(Ulid::from_string(&row.id)?);
				let url = Url::parse(&row.url)?;
				let title = row.title;
				Ok(Some(Feed { id, url, title }))
			},
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(Box::new(e)),
		}
	}

	fn add_entry(&mut self, entry: &Entry, feed_id: &FeedId) -> Result<()> {
		let entry_id = entry.id.to_string();
		let feed_id = feed_id.to_string();
		let article_date: i64 = entry.article_date.duration_since(SystemTime::UNIX_EPOCH).unwrap().as_millis().try_into().unwrap();
		let entry_url = entry.url.clone().map(|url| url.to_string());
		self.async_util.run_blocking(|| async {
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
				.await
		} )?;
		Ok(())
	}

	fn get_entry(&self, id: &EntryId) -> Result<Entry> {
		let entry_id = id.to_string();
		let row = self.async_util.run_blocking(|| async {
			sqlx::query!("
					SELECT
						id, feed_id, internal_id, fetch_index, article_date, title, url
					FROM entries
					WHERE id = ?;",
					entry_id,
				)
				.fetch_one(&self.pool)
				.await
		} )?;
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

	fn get_entries_for_feed(&self, feed_id: &FeedId) -> impl IntoIterator<Item = Result<Entry>> {
		let feed_id = feed_id.to_string();
		// TODO: Maybe do paging later. Or figure out how to stream from sqlx.
		let rows = self.async_util.run_blocking(|| async {
			sqlx::query!("
					SELECT
						id, feed_id, internal_id, fetch_index, article_date, title, url
					FROM entries
					WHERE feed_id = ?;",
					feed_id,
				)
				.fetch_all(&self.pool)
				.await
		} );
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

	fn get_and_increment_fetch_index(&mut self) -> Result<u32> {
		let row = self.async_util.run_blocking(|| async {
			sqlx::query!("
					UPDATE metadata
					SET fetch_index = fetch_index + 1
					RETURNING fetch_index"
				)
				.fetch_one(&self.pool)
				.await
		} )?;
		let index = (row.fetch_index - 1).try_into()?;
		Ok(index)
	}

	fn add_user(&mut self, user: &User) -> Result<()> {
		let user_id = user.id.to_string();
		self.async_util.run_blocking(|| async {
			sqlx::query!("
					INSERT INTO users (
						id, name, password_hash
					) VALUES ( ?, ?, ? )",
					user_id,
					user.name,
					user.password_hash,
				)
				.execute(&self.pool)
				.await
		} )?;
		Ok(())
	}

	fn get_user_by_name(&self, user_name: &str) -> Result<Option<User>> {
		let row_result = self.async_util.run_blocking(|| async {
			sqlx::query!("
					SELECT
						id, name, password_hash
					FROM users
					WHERE name = ?;",
					user_name)
				.fetch_one(&self.pool)
				.await
		} );
		match row_result {
			Ok(row) => {
				let id = UserId(Ulid::from_string(&row.id)?);
				Ok(Some(User {
					id,
					name: row.name,
					password_hash: row.password_hash
				} ) )
			},
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(Box::new(e)),
		}
	}
}
