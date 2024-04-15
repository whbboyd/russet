pub mod entries;
pub mod feeds;
pub mod model;
pub mod user;

use crate::feed::RussetFeedReader;
use crate::Result;

pub struct RussetDomainService<Persistence>
where Persistence: std::fmt::Debug {
	persistence: Persistence,
	readers: Vec<Box<dyn RussetFeedReader>>,
	pepper: Vec<u8>,
}
impl <Persistence> RussetDomainService<Persistence>
where Persistence: std::fmt::Debug {
	pub fn new(
		persistence: Persistence,
		readers: Vec<Box<dyn RussetFeedReader>>,
		pepper: Vec<u8>,
	) -> Result<RussetDomainService<Persistence>> {
		Ok(RussetDomainService {
			persistence,
			readers,
			pepper,
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
