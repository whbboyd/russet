use crate::model::{ FeedId, Pagination, UserId };
use crate::persistence::RussetFeedPersistenceLayer;
use crate::persistence::sql::SqlDatabase;
use crate::persistence::model::{ Feed, FeedCheck, WriteFeedCheck };
use crate::Result;
use reqwest::Url;
use ulid::Ulid;

impl RussetFeedPersistenceLayer for SqlDatabase {

	#[tracing::instrument]
	async fn add_feed(&self, feed: &Feed) -> Result<()> {
		let feed_id = feed.id.to_string();
		let feed_url = feed.url.to_string();
		sqlx::query!("
				INSERT INTO feeds (
					id, url, title
				) VALUES ( ?, ?, ? )",
				feed_id,
				feed_url,
				feed.title,
			)
			.execute(&self.pool)
			.await?;
		Ok(())
	}

	#[tracing::instrument]
	async fn get_feeds(&self) -> Vec<Result<Feed>> {
		// TODO: Maybe do paging later. Or figure out how to stream from sqlx.
		let rows = sqlx::query!("
				SELECT
					id, url, title
				FROM feeds;"
			)
			.fetch_all(&self.pool)
			.await;
		let rv: Vec<Result<Feed>> = match rows {
			Ok(rows) => {
				rows.into_iter()
					.map(|row| {
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

	#[tracing::instrument]
	async fn get_feed(&self, id: &FeedId) -> Result<Feed> {
		let feed_id = id.to_string();
		let row = sqlx::query!("
				SELECT
					url, title
				FROM feeds
				WHERE id = ?;",
				feed_id,
			)
			.fetch_one(&self.pool)
			.await?;
		let id = FeedId(id.0.clone());
		let url = Url::parse(&row.url)?;
		let title = row.title;
		Ok(Feed { id, url, title })
	}

	#[tracing::instrument]
	async fn get_feed_by_url(&self, url: &Url) -> Result<Option<Feed>> {
		let feed_url = url.to_string();
		let row_result = sqlx::query!("
				SELECT
					id, url, title
				FROM feeds
				WHERE url = ?;",
				feed_url)
			.fetch_one(&self.pool)
			.await;
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

	#[tracing::instrument]
	async fn get_subscribed_feeds(&self, user_id: &UserId) -> Vec<Result<Feed>> {
		let user_id = user_id.to_string();
		let rows = sqlx::query!("
				SELECT
					f.id, f.url, f.title
				FROM feeds AS f
				INNER JOIN subscriptions AS s
					ON f.id = s.feed_id
				WHERE s.user_id = ?;",
				user_id,
			)
			.fetch_all(&self.pool)
			.await;
		let rv: Vec<Result<Feed>> = match rows {
			Ok(rows) => {
				rows.into_iter()
					.map(|row| {
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

	#[tracing::instrument]
	async fn add_feed_check(&self, feed_check: WriteFeedCheck) -> Result<FeedCheck> {
		let mut tx = self.pool.begin().await?;
		let next_fetch_index = sqlx::query!("
				SELECT
					MAX(id) AS id
				FROM feed_checks;")
			.fetch_one(&mut *tx)
			.await?
			.id
			.unwrap_or(0)
			+ 1;
		let feed_id = feed_check.feed_id.to_string();
		let check_time: i64 = feed_check.check_time.try_into()?;
		let next_check_time: i64 = feed_check.next_check_time.try_into()?;
		sqlx::query!("
				INSERT INTO feed_checks (
					id, feed_id, check_time, next_check_time, etag
				) VALUES ( ?, ?, ?, ?, ? )",
				next_fetch_index,
				feed_id,
				check_time,
				next_check_time,
				feed_check.etag,
			)
			.execute(&mut *tx)
			.await?;
		tx.commit().await?;
		Ok(FeedCheck::from_write_feed_check(next_fetch_index.try_into()?, feed_check))
	}

	#[tracing::instrument]
	async fn get_feed_checks(
		&self,
		feed_id: &FeedId,
		pagination: &Pagination,
	) -> Vec<Result<FeedCheck>> {
		let feed_id = feed_id.to_string();
		let page_size: i64 = match pagination.page_size.try_into() {
			Ok(i) => i,
			Err(e) => return vec![Err(e.into())]
		};
		let page_offset: i64 = match (pagination.page_num * pagination.page_size).try_into() {
			Ok(i) => i,
			Err(e) => return vec![Err(e.into())]
		};
		let rows = sqlx::query!("
				SELECT
					id, feed_id, check_time, next_check_time, etag
				FROM feed_checks
				WHERE feed_id = ?
				ORDER BY id DESC
				LIMIT ?
				OFFSET ?;",
				feed_id,
				page_size,
				page_offset,
			)
			.fetch_all(&self.pool)
			.await;
		let rv: Vec<Result<FeedCheck>> = match rows {
			Ok(rows) => {
				rows.into_iter().map(|row| {
					let id: u64 = row.id.try_into()?;
					let feed_id = FeedId(Ulid::from_string(&row.feed_id)?);
					Ok(FeedCheck {
						id,
						feed_id,
						check_time: row.check_time.try_into()?,
						next_check_time: row.next_check_time.try_into()?,
						etag: row.etag,
					} )
				} )
					.collect()
			},
			Err(e) => vec![Err(Box::new(e))],
		};
		rv
	}
}
