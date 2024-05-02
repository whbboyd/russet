use argon2::{ Argon2, PasswordHasher, PasswordVerifier };
use argon2::password_hash::Error::Password;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::SaltString;
use base32ct::{ Base32Unpadded, Encoding };
use crate::domain::RussetDomainService;
use crate::model::{ FeedId, Timestamp, UserId };
use crate::persistence::model::{ PasswordHash, Session, SessionToken, User };
use crate::persistence::RussetUserPersistenceLayer;
use crate::Err;
use crate::Result;
use getrandom::getrandom;
use std::time::{ Duration, SystemTime };
use tracing::info;
use ulid::Ulid;

impl <Persistence> RussetDomainService<Persistence>
where Persistence: RussetUserPersistenceLayer {

	pub async fn login_user(
		&self,
		user_name: String,
		plaintext_password: String,
		permanent_session: bool,
	) -> Result<Option<Session>> {
		let password_hash = Argon2::new_with_secret(
				self.pepper.as_slice(),
				argon2::Algorithm::Argon2id,
				argon2::Version::V0x13,
				argon2::Params::DEFAULT,
			)?;
		let password_bytes = plaintext_password.into_bytes();
		let user = self.persistence.get_user_by_name(&user_name).await?;
		match user {
			Some(user) => {
				let parsed_hash = argon2::PasswordHash::new(&user.password_hash.0)?;
				match password_hash.verify_password(&password_bytes, &parsed_hash) {
					Ok(_) => {
						let token = Self::generate_token()?;
						let session_duration = if permanent_session {
							// Gigasecond is > 30 years. Should be long enough.
							Duration::from_secs(1_000_000_000)
						} else {
							// Otherwise, we'll expire in a week (but set a
							// session cookie)
							Duration::from_secs(7 * 24 * 60 * 60)
						};
						let expiration = Timestamp::new(SystemTime::now() + session_duration);
						let session = Session {
							token,
							user_id: user.id,
							expiration,
						};
						self.persistence.add_session(&session).await?;
						info!("Successfully logged in {:?} ({:?})", user.name, session);
						Ok(Some(session))
					},
					Err(Password) => {
						info!("Bad password for {:?}", user.name);
						Ok(None)
					},
					Err(e) => Err(Box::new(e)),
				}
			}
			None => {
				// Hash the password anyway to resist user enumeration via side channels
				let parsed_hash = argon2::PasswordHash::new("$argon2id$v=19$m=19456,t=2,p=1$DFhnniX1Kn3JoEKD5e9qbQ$IxgxUYNYPTvPTjez280uFJh166f+eNkCXntlVe5NaZQ").unwrap();
				let _ = password_hash.verify_password(&password_bytes, &parsed_hash);
				info!("User {:?} not found", user_name);
				Ok(None)
			}
		}
	}

	pub async fn add_user(&self, user_name: &str, plaintext_password: &str) -> Result<()> {
		if let Some(user) = self.persistence.get_user_by_name(&user_name).await? {
			return Err(format!("User {} ({}) already exists", user.name, user.id.to_string()).into());
		}
		let password_hash = self.hash_password(plaintext_password)?;
		let user = User {
			id: UserId(Ulid::new()),
			name: user_name.to_string(),
			password_hash,
		};
		self.persistence.add_user(&user).await?;
		Ok(())
	}

	pub async fn set_user_password(
		&self,
		user_name: &str,
		plaintext_password: &str,
	) -> Result<()> {
		let user = self.persistence
			.get_user_by_name(user_name)
			.await?
			.ok_or_else(|| -> Err { format!("No such user {user_name}").into() })?;
		let password_hash = self.hash_password(plaintext_password)?;
		let user = User { password_hash, ..user };
		self.persistence.update_user(&user).await?;
		Ok(())
	}

	pub async fn delete_user(&self, user_name: &str) -> Result<()> {
		let user = self.persistence
			.get_user_by_name(user_name)
			.await?
			.ok_or_else(|| -> Err { format!("No such user {user_name}").into() })?;
		self.persistence.delete_user(&user.id).await?;
		Ok(())
	}

	pub async fn delete_user_sessions(&self, user_name: &str) -> Result<()> {
		let user = self.persistence
			.get_user_by_name(user_name)
			.await?
			.ok_or_else(|| -> Err { format!("No such user {user_name}").into() })?;
		self.persistence.delete_sessions_for_user(&user.id).await?;
		Ok(())
	}
	
	pub async fn  cleanup_expired_sessions(&self) -> Result<()> {
		let expiry = Timestamp::new(SystemTime::now());
		self.persistence.delete_expired_sessions(&expiry).await
	}

	pub async fn auth_user(&self, token: &str) -> Result<Option<User>> {
		match self.persistence.get_user_by_session(&token).await? {
			Some((user, session)) => {
				if session.expiration.0 < SystemTime::now() {
					// If the session expired, clear it and fail the auth
					info!("Session {:?} expired, removing…", session.token);
					self.persistence.delete_session(&session.token.0).await?;
					Ok(None)
				} else {
					Ok(Some(user))
				}
			},
			None => Ok(None)
		}
	}

	pub async fn subscribe(&self, user_id: &UserId, feed_id: &FeedId) -> Result<()> {
		self.persistence.add_subscription(user_id, feed_id).await
	}

	pub async fn unsubscribe(&self, user_id: &UserId, feed_id: &FeedId) -> Result<()> {
		self.persistence.remove_subscription(user_id, feed_id).await
	}

	fn generate_token() -> Result<SessionToken> {
		let mut bytes = [0u8; 32];
		getrandom(&mut bytes)?;
		Ok(SessionToken(Base32Unpadded::encode_string(&bytes)))
	}

	fn hash_password(&self, plaintext_password: &str) -> Result<PasswordHash> {
		let password_hasher = Argon2::new_with_secret(
				self.pepper.as_slice(),
				argon2::Algorithm::Argon2id,
				argon2::Version::V0x13,
				argon2::Params::DEFAULT,
			)?;
		let salt = SaltString::generate(&mut OsRng);
		Ok(PasswordHash(
			password_hasher
				.hash_password(plaintext_password.as_bytes(), &salt)?
				.to_string()
		))
	}
}
