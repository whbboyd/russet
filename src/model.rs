/// Utility types

use clap::ValueEnum;
use crate::{ Err, Result };
use serde::Deserialize;
use std::ops::Deref;
use std::time::SystemTime;
use ulid::Ulid;

#[derive(Clone, Copy, Deserialize)]
pub struct Timestamp(pub SystemTime);
impl Timestamp {
	pub fn new(time: SystemTime) -> Timestamp {
		Timestamp(time)
	}
}
impl std::fmt::Debug for Timestamp {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.write_fmt(format_args!("{}", "some timestamp"))
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
