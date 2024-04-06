pub mod entries;
pub mod feeds;
pub mod user;

use crate::feed::RussetFeedReader;
use crate::persistence::RussetPersistenceLayer;
use crate::Result;

pub struct RussetDomainService<Persistence>
where Persistence: RussetPersistenceLayer {
	persistence: Persistence,
	readers: Vec<Box<dyn RussetFeedReader + Send + Sync>>,
	pepper: Vec<u8>,
}
impl <Persistence> RussetDomainService<Persistence>
where Persistence: RussetPersistenceLayer {
	pub fn new(
		persistence: Persistence,
		readers: Vec<Box<dyn RussetFeedReader + Send + Sync>>,
		pepper: Vec<u8>,
	) -> Result<RussetDomainService<Persistence>> {
		Ok(RussetDomainService {
			persistence,
			readers,
			pepper,
		} )
	}
}
