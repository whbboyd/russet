use argon2::{ Argon2, PasswordHash, PasswordHasher, PasswordVerifier };
use argon2::password_hash::Error::Password;
use argon2::password_hash::SaltString;
use argon2::password_hash::rand_core::OsRng;
use crate::domain::RussetDomainService;
use crate::Result;
use crate::persistence::RussetPersistenceLayer;
use crate::persistence::model::{ User, UserId };
use ulid::Ulid;

impl <'pepper, Persistence> RussetDomainService<Persistence>
where Persistence: RussetPersistenceLayer {

	pub async fn login_user(&mut self, user_name: String, plaintext_password: String) -> Result<Option<String>> {
		let password_hash = Argon2::new_with_secret(
				self.pepper.as_slice(),
				argon2::Algorithm::Argon2id,
				argon2::Version::V0x13,
				argon2::Params::DEFAULT,
			)?;
		let password_bytes = plaintext_password.into_bytes();
		match self.persistence.get_user_by_name(&user_name).await? {
			Some(user) => {
				let parsed_hash = PasswordHash::new(&user.password_hash)?;
				match password_hash.verify_password(&password_bytes, &parsed_hash) {
					Ok(_) => {
						todo!()
					},
					Err(Password) => Ok(None),
					Err(e) => Err(Box::new(e)),
				}
			}
			None => {
				// Hash the password anyway to resist user enumeration via side channels
				let parsed_hash = PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$DFhnniX1Kn3JoEKD5e9qbQ$IxgxUYNYPTvPTjez280uFJh166f+eNkCXntlVe5NaZQ").unwrap();
				let _ = password_hash.verify_password(&password_bytes, &parsed_hash);
				Ok(None)
			}
		}
	}

	pub async fn add_user(&mut self, user_name: String, plaintext_password: String) -> Result<()> {
		if let Some(user) = self.persistence.get_user_by_name(&user_name).await? {
			return Err(format!("User {} ({}) already exists", user.name, user.id.to_string()).into());
		}
		let password_hash = Argon2::new_with_secret(
				self.pepper.as_slice(),
				argon2::Algorithm::Argon2id,
				argon2::Version::V0x13,
				argon2::Params::DEFAULT,
			)?;
		let salt = SaltString::generate(&mut OsRng);
		let password_hash = password_hash.hash_password(plaintext_password.as_bytes(), &salt)?.to_string();
		let user = User {
			id: UserId(Ulid::new()),
			name: user_name,
			password_hash,
		};
		self.persistence.add_user(&user).await?;
		Ok(())
	}
}
