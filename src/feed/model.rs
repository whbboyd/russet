use crate::model::Timestamp;
use reqwest::Url;

#[derive(Debug)]
pub struct Feed {
	pub title: String,
	pub entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entry {
	pub internal_id: String,
	pub url: Option<Url>,
	pub article_date: Timestamp,
	pub title: String,
}
