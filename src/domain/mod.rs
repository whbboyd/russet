pub mod entries;
pub mod feeds;
pub mod model;
pub mod user;

use crate::feed::RussetFeedReader;
use crate::Result;
use std::time::Duration;

const MIN_CHECK_INTERVAL: Duration = Duration::from_secs(60);

pub struct RussetDomainService<Persistence>
where Persistence: std::fmt::Debug {
	persistence: Persistence,
	readers: Vec<Box<dyn RussetFeedReader>>,
	pepper: Vec<u8>,
	pub feed_check_interval: Duration,
}
impl <Persistence> RussetDomainService<Persistence>
where Persistence: std::fmt::Debug {
	pub fn new(
		persistence: Persistence,
		readers: Vec<Box<dyn RussetFeedReader>>,
		pepper: Vec<u8>,
		feed_check_interval: Duration,
	) -> Result<RussetDomainService<Persistence>> {
		if feed_check_interval < MIN_CHECK_INTERVAL {
			let interval = feed_check_interval.as_secs_f64();
			let min_interval = MIN_CHECK_INTERVAL.as_secs_f64();
			return Err(format!("Feed check interval {interval}s is \
				less than minimum allowed interval of {min_interval}s").into());
		}
		Ok(RussetDomainService {
			persistence,
			readers,
			pepper,
			feed_check_interval,
		} )
	}
}
impl <Persistence> std::fmt::Debug for RussetDomainService<Persistence>
where Persistence: std::fmt::Debug {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("RussetPersistenceLayer")
			.field("persistence", &self.persistence)
			.field("readers", &self.readers)
			.field("pepper", &"<redacted>")
			.finish()
	}
}
