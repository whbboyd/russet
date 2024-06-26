use crate::model::{ FeedId, UserId };
use crate::persistence::RussetFeedPersistenceLayer;
use crate::persistence::sql::SqlDatabase;
use crate::persistence::model::Feed;
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
}
