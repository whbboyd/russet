/// Utility types

use clap::ValueEnum;
use crate::{ Err, Result };
use serde::Deserialize;
use std::ops::Deref;
use std::time::{ Duration, SystemTime };
use ulid::Ulid;

#[derive(Clone, Copy, Deserialize, Eq, PartialEq, Ord, PartialOrd)]
pub struct Timestamp(pub SystemTime);
impl Timestamp {
	pub fn new(time: SystemTime) -> Timestamp {
		Timestamp(time)
	}
	pub fn now() -> Timestamp {
		Timestamp(SystemTime::now())
	}
	/// Duration between now and the given [timestamp]. Zero if the given
	/// [timestamp] is in the past relative to the current system clock.
	pub fn until(timestamp: Timestamp) -> Duration {
		timestamp.0.duration_since(Self::now().0).unwrap_or(Duration::ZERO)
	}
}
impl std::fmt::Debug for Timestamp {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let unix_secs = match self.0.duration_since(SystemTime::UNIX_EPOCH) {
			Ok(duration) => duration.as_secs_f64(),
			Err(backwards) => -backwards.duration().as_secs_f64(),
		};
		f.write_fmt(format_args!("{}", unix_secs))
	}
}
impl std::ops::Add<Duration> for Timestamp {
	type Output = Timestamp;
	fn add(self, rhs: Duration) -> Timestamp {
		Timestamp(self.0 + rhs)
	}
}
impl std::ops::Sub<Timestamp> for Timestamp {
	type Output = Result<Duration>;
	fn sub(self, rhs: Timestamp) -> Result<Duration> {
		Ok(self.0.duration_since(rhs.0)?)
	}
}

#[derive(Clone, Copy, Deserialize, Debug)]
pub struct Pagination {
	pub page_num: usize,
	pub page_size: usize,
}

#[derive(Clone, Copy, Deserialize, Eq, Hash, PartialEq)]
pub struct FeedId(pub Ulid);
impl Deref for FeedId { type Target = Ulid; fn deref(&self) -> &Self::Target { &self.0 } }
impl std::fmt::Debug for FeedId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("\"{}\"", &self.to_string()))
	}
}
#[derive(Clone, Copy, Deserialize)]
pub struct EntryId(pub Ulid);
impl Deref for EntryId{ type Target = Ulid; fn deref(&self) -> &Self::Target { &self.0 } }
impl std::fmt::Debug for EntryId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("\"{}\"", &self.to_string()))
	}
}
#[derive(Clone, Copy, Deserialize, PartialEq, Eq)]
pub struct UserId(pub Ulid);
impl Deref for UserId{ type Target = Ulid; fn deref(&self) -> &Self::Target { &self.0 } }
impl std::fmt::Debug for UserId {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("\"{}\"", &self.to_string()))
	}
}
#[derive(Clone, Copy, Deserialize, Debug, PartialEq, Eq, ValueEnum)]
pub enum UserType {
	Sysop,
	Member,
}
impl TryFrom<String> for UserType {
	type Error = Err;
	fn try_from(str: String) -> Result<UserType> {
		match str.as_str() {
			"Sysop" => Ok(UserType::Sysop),
			"Member" => Ok(UserType::Member),
			_ => Err(format!("Unrecognized value {str} (must be one of \"Sysop\", \"Member\")").into()),
		}
	}
}
impl Into<String> for UserType {
	fn into(self) -> String {
		match self {
			UserType::Sysop => "Sysop".to_string(),
			UserType::Member => "Member".to_string(),
		}
	}
}
