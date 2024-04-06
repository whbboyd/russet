use crate::persistence::model::{ User, UserId };
use crate::persistence::RussetUserPersistenceLayer;
use crate::persistence::sql::SqlDatabase;
use crate::Result;
use ulid::Ulid;

impl RussetUserPersistenceLayer for SqlDatabase {

	async fn add_user(&mut self, user: &User) -> Result<()> {
		let user_id = user.id.to_string();
		sqlx::query!("
				INSERT INTO users (
					id, name, password_hash
				) VALUES ( ?, ?, ? )",
				user_id,
				user.name,
				user.password_hash,
			)
			.execute(&self.pool)
			.await?;
		Ok(())
	}

	async fn get_user_by_name(&self, user_name: &str) -> Result<Option<User>> {
		let row_result = sqlx::query!("
				SELECT
					id, name, password_hash
				FROM users
				WHERE name = ?;",
				user_name)
			.fetch_one(&self.pool)
			.await;
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
