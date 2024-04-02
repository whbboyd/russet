use reqwest::Url;
use std::time::SystemTime;

#[derive(Debug)]
pub struct Feed {
	pub title: String,
	pub url: Url,
	pub entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entry {
	pub internal_id: String,
	pub url: Option<Url>,
	pub fetch_index: u32,
	pub article_date: SystemTime,
	pub title: String,
}
