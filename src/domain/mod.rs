pub mod feeds;
pub mod user;

use argon2::Argon2;
use crate::feed::RussetFeedReader;
use crate::persistence::RussetPersistenceLayer;
use crate::Result;

pub struct RussetDomainService<'pepper, Persistence>
where Persistence: RussetPersistenceLayer {
	persistence: Persistence,
	readers: Vec<Box<dyn RussetFeedReader>>,
	password_hash: Argon2<'pepper>,
}
impl <'pepper, Persistence> RussetDomainService<'pepper, Persistence>
where Persistence: RussetPersistenceLayer {
	pub fn new(
		persistence: Persistence,
		readers: Vec<Box<dyn RussetFeedReader>>,
		pepper: &'pepper [u8],
	) -> Result<RussetDomainService<'pepper, Persistence>> {
		Ok(RussetDomainService {
			persistence,
			readers,
			password_hash: Argon2::new_with_secret(
				&pepper,
				argon2::Algorithm::Argon2id,
				argon2::Version::V0x13,
				argon2::Params::DEFAULT,
			)?,
		} )
	}
}
