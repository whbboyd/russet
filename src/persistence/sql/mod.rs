pub mod entry;
pub mod feed;
pub mod user;

use crate::persistence::RussetPersistenceLayer;
use crate::Result;
use sqlx::{ Pool, Sqlite };
use std::error::Error;
use std::path::Path;

#[derive(Debug)]
pub struct SqlDatabase {
	pool: Pool<Sqlite>,
}
impl SqlDatabase {
	pub async fn new(db_path: &Path) -> Result<SqlDatabase> {
		let path = db_path
			.to_str()
			.ok_or::<Box<dyn Error + Send + Sync + 'static>>("db_path is not valid UTF-8".into())?;
		let pool = Pool::<Sqlite>::connect(path).await?;
		sqlx::migrate!("db/migrations/").run(&pool).await?;
		Ok(SqlDatabase { pool })
	}
}

impl RussetPersistenceLayer for SqlDatabase { }
