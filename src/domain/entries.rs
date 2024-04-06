use crate::persistence::model::Entry;
use crate::domain::RussetDomainService;
use crate::Result;
use crate::persistence::RussetPersistenceLayer;

impl <Persistence> RussetDomainService<Persistence>
where Persistence: RussetPersistenceLayer + std::fmt::Debug {
}
