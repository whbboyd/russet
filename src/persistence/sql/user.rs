use crate::model::{ FeedId, Timestamp, UserId };
use crate::persistence::model::{ PasswordHash, Session, SessionToken, User };
use crate::persistence::RussetUserPersistenceLayer;
use crate::persistence::sql::SqlDatabase;
use crate::Result;
use ulid::Ulid;

impl RussetUserPersistenceLayer for SqlDatabase {

	#[tracing::instrument]
	async fn get_user(&self, user_id: &UserId) -> Result<User> {
		let user_id = user_id.to_string();
		let row = sqlx::query!("
				SELECT
					id, name, password_hash, user_type
				FROM users
				WHERE id = ?;",
				user_id)
			.fetch_one(&self.pool)
			.await?;
		let id = UserId(Ulid::from_string(&row.id)?);
		let password_hash = PasswordHash(row.password_hash);
		Ok(User {
			id,
			name: row.name,
			password_hash,
			user_type: row.user_type.try_into()?,
		} )
	}

	#[tracing::instrument]
	async fn add_user(&self, user: &User) -> Result<()> {
		let user_id = user.id.to_string();
		let password_hash = &user.password_hash.0;
		let user_type: String = user.user_type.into();
		sqlx::query!("
				INSERT INTO users (
					id, name, password_hash, user_type
				) VALUES ( ?, ?, ?, ? );",
				user_id,
				user.name,
				password_hash,
				user_type,
			)
			.execute(&self.pool)
			.await?;
		Ok(())
	}

	#[tracing::instrument]
	async fn update_user(&self, user: &User) -> Result<()> {
		let user_id = user.id.to_string();
		let password_hash = &user.password_hash.0;
		sqlx::query!("
				UPDATE users SET
					name = ?,
					password_hash = ?
				WHERE id = ?;",
				user.name,
				password_hash,
				user_id,
			)
			.execute(&self.pool)
			.await?;
		Ok(())
	}

	#[tracing::instrument]
	async fn delete_user(&self, user_id: &UserId) -> Result<()> {
		let user_id = user_id.to_string();
		sqlx::query!("
				DELETE FROM sessions
				WHERE user_id = ?;
				DELETE FROM user_entry_settings
				WHERE user_id = ?;
				DELETE FROM subscriptions
				WHERE user_id = ?;
				DELETE FROM users
				WHERE id = ?",
				user_id,
				user_id,
				user_id,
				user_id,
			)
			.execute(&self.pool)
			.await?;
		Ok(())
	}

	#[tracing::instrument]
	async fn get_user_by_name(&self, user_name: &str) -> Result<Option<User>> {
		let row_result = sqlx::query!("
				SELECT
					id, name, password_hash, user_type
				FROM users
				WHERE name = ?;",
				user_name)
			.fetch_one(&self.pool)
			.await;
		match row_result {
			Ok(row) => {
				let id = UserId(Ulid::from_string(&row.id)?);
				let password_hash = PasswordHash(row.password_hash);
				Ok(Some(User {
					id,
					name: row.name,
					password_hash,
					user_type: row.user_type.try_into()?,
				} ) )
			},
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(Box::new(e)),
		}
	}

	#[tracing::instrument]
	async fn add_session(&self, session: &Session) -> Result<()> {
		let user_id = session.user_id.to_string();
		let expiration: i64 = session.expiration.clone().try_into()?;
		sqlx::query!("
				INSERT INTO sessions (
					token, user_id, expiration
				) VALUES ( ?, ?, ? )",
				session.token.0,
				user_id,
				expiration,
			)
			.execute(&self.pool)
			.await?;
		Ok(())
	}

	#[tracing::instrument]
	async fn get_user_by_session(&self, session_token: &str) -> Result<Option<(User, Session)>> {
		let row_result = sqlx::query!("
				SELECT
					users.id, users.name, users.password_hash, users.user_type,
					sessions.expiration
				FROM users
				JOIN sessions
				ON users.id = sessions.user_id
				WHERE sessions.token = ?;",
				session_token)
			.fetch_one(&self.pool)
			.await;
		match row_result {
			Ok(row) => {
				let user_id = UserId(Ulid::from_string(&row.id)?);
				let password_hash = PasswordHash(row.password_hash);
				Ok(Some((
					User {
						id: user_id.clone(),
						name: row.name,
						password_hash,
						user_type: row.user_type.try_into()?,
					},
					Session {
						token: SessionToken(session_token.to_string()),
						user_id,
						expiration: row.expiration.into(),
					}
				) ) )
			},
			Err(sqlx::Error::RowNotFound) => Ok(None),
			Err(e) => Err(Box::new(e)),
		}
	}

	#[tracing::instrument]
	async fn delete_session(&self, session_token: &str) -> Result<()> {
		let rows = sqlx::query!("
				DELETE FROM sessions
				WHERE token = ?;",
				session_token,
			)
			.execute(&self.pool)
			.await?
			.rows_affected();
		if rows != 1 {
			Err(format!("Deleted {} (not 1) sessions with token {:?}", rows, session_token).into())
		} else {
			Ok(())
		}
	}

	#[tracing::instrument]
	async fn delete_expired_sessions(&self, expiration: &Timestamp) -> Result<()> {
		let expiration: i64 = expiration.clone().try_into()?;
		sqlx::query!("
				DELETE FROM sessions
				wHERE expiration < ?;",
				expiration,
			)
			.execute(&self.pool)
			.await?
			.rows_affected();
		Ok(())
	}

	#[tracing::instrument]
	async fn delete_sessions_for_user(&self, user_id: &UserId) -> Result<u32> {
		let user_id = user_id.to_string();
		let rows = sqlx::query!("
				DELETE FROM sessions
				WHERE user_id = ?;",
				user_id,
			)
			.execute(&self.pool)
			.await?
			.rows_affected()
			.try_into()?;
		Ok(rows)
	}

	#[tracing::instrument]
	async fn add_subscription(&self, user_id: &UserId, feed_id: &FeedId) -> Result<()> {
		let feed_id = feed_id.to_string();
		let user_id = user_id.to_string();
		sqlx::query!("
				INSERT INTO subscriptions(
					user_id, feed_id
				) VALUES ( ?, ? )",
				user_id,
				feed_id,
			)
			.execute(&self.pool)
			.await?;
		Ok(())
	}

	#[tracing::instrument]
	async fn remove_subscription(&self, user_id: &UserId, feed_id: &FeedId) -> Result<()> {
		let feed_id = feed_id.to_string();
		let user_id = user_id.to_string();
		sqlx::query!("
				DELETE FROM subscriptions
				WHERE user_id = ? AND feed_id = ?
				",
				user_id,
				feed_id,
			)
			.execute(&self.pool)
			.await?;
		Ok(())
	}
}
