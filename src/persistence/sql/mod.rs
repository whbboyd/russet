pub mod entry;
pub mod feed;
pub mod user;

use crate::async_util::AsyncUtil;
use crate::persistence::RussetPersistenceLayer;
use crate::Result;
use sqlx::{ Pool, Sqlite };
use std::error::Error;
use std::path::Path;
use std::sync::Arc;

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

impl RussetPersistenceLayer for SqlDatabase { }
