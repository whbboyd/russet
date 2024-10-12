pub mod entries;
pub mod feeds;
pub mod model;
pub mod user;

use crate::feed::RussetFeedReader;
use crate::Result;
use std::time::Duration;

pub struct RussetDomainService<Persistence>
where Persistence: std::fmt::Debug {
	persistence: Persistence,
	readers: Vec<Box<dyn RussetFeedReader>>,
	pepper: Vec<u8>,
	min_feed_check_interval: Duration,
	pub default_feed_check_interval: Duration,
	max_feed_check_interval: Duration,
	disable_logins: bool,
}
impl <Persistence> RussetDomainService<Persistence>
where Persistence: std::fmt::Debug {
	pub fn new(
		persistence: Persistence,
		readers: Vec<Box<dyn RussetFeedReader>>,
		pepper: Vec<u8>,
		min_feed_check_interval: Duration,
		default_feed_check_interval: Duration,
		max_feed_check_interval: Duration,
		disable_logins: bool,
	) -> Result<RussetDomainService<Persistence>> {
		if min_feed_check_interval > default_feed_check_interval {
			let min_interval = min_feed_check_interval.as_secs_f64();
			let default_interval = default_feed_check_interval.as_secs_f64();
			return Err(format!("Min check interval ${min_interval}s is greater \
				than default interval ${default_interval}s").into());
		}
		if default_feed_check_interval > max_feed_check_interval {
			let default_interval = default_feed_check_interval.as_secs_f64();
			let max_interval = max_feed_check_interval.as_secs_f64();
			return Err(format!("Default check interval ${default_interval}s is \
				greater than max interval ${max_interval}s").into());
		}
		Ok(RussetDomainService {
			persistence,
			readers,
			pepper,
			min_feed_check_interval,
			default_feed_check_interval,
			max_feed_check_interval,
			disable_logins,
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
			.field("min_feed_check_interval", &self.min_feed_check_interval)
			.field("default_feed_check_interval", &self.default_feed_check_interval)
			.field("max_feed_check_interval", &self.max_feed_check_interval)
			.field("disable_logins", &self.disable_logins)
			.finish()
	}
}
