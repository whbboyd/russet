pub mod entry;
pub mod feed;
pub mod user;

use crate::Err;
use crate::model::Timestamp;
use crate::persistence::RussetPersistenceLayer;
use crate::Result;
use sqlx::{ Pool, Sqlite };
use std::path::Path;
use std::time::{ Duration, SystemTime };

#[derive(Debug)]
pub struct SqlDatabase {
	pool: Pool<Sqlite>,
}
impl SqlDatabase {
	pub async fn new(db_path: &Path) -> Result<SqlDatabase> {
		let path = db_path
			.to_str()
			.ok_or::<Err>("db_path is not valid UTF-8".into())?;
		let pool = Pool::<Sqlite>::connect(path).await?;
		sqlx::migrate!("db/migrations/").run(&pool).await?;
		Ok(SqlDatabase { pool })
	}
}

impl RussetPersistenceLayer for SqlDatabase { }

impl From<i64> for Timestamp {
	fn from(value: i64) -> Timestamp {
		let duration = Duration::from_millis(value.abs_diff(0));
		let time = if value >= 0 {
			SystemTime::UNIX_EPOCH + duration
		} else {
			SystemTime::UNIX_EPOCH - duration
		};
		Timestamp(time)
	}
}

impl TryFrom<Timestamp> for i64 {
	type Error = Err;
	fn try_from(value: Timestamp) -> Result<i64> {
		Ok(value.0.duration_since(SystemTime::UNIX_EPOCH).map_or_else(
			// Err case: value is prior to epoch, so do the comparison in the
			// other direction and negate the result
			|_| SystemTime::UNIX_EPOCH
					.duration_since(value.0)
					.expect("duration_since should only return Err if the SystemTimes are in reverse order")
					.as_millis()
					.try_into()
					.map(|i: i64| -i),
			// Ok case: just pull out the milliseconds
			|duration| duration.as_millis().try_into(),
		)?)
	}
}
