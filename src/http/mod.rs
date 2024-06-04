use axum::middleware::map_response;
use axum::response::Response;
use axum::Router;
use axum::routing::{ any, get, post };
use crate::domain::RussetDomainService;
use crate::http::session::AuthenticatedUser;
use crate::persistence::RussetPersistenceLayer;
use serde::Deserialize;
use std::sync::Arc;
use tokio::sync::Semaphore;
use tower::limit::GlobalConcurrencyLimitLayer;
use tower_http::compression::CompressionLayer;

mod entry;
pub mod error;
mod feed;
mod login;
mod root;
mod session;
mod static_routes;
mod subscribe;
mod user;

pub fn russet_router<Persistence>(
	global_concurrent_limit: u32,
	login_concurrent_limit: u32,
) -> Router<AppState<Persistence>>
where Persistence: RussetPersistenceLayer {
	let global_limit_semaphore = Arc::new(Semaphore::new(global_concurrent_limit.try_into().unwrap()));
	let login_limit_sempahore = Arc::new(Semaphore::new(login_concurrent_limit.try_into().unwrap()));
	Router::new()
		.route("/login", post(login::login_user))
		.layer(GlobalConcurrencyLimitLayer::with_semaphore(login_limit_sempahore))
		.route("/login", get(login::login_page))
		.route("/styles.css", get(static_routes::styles))
		.route("/", get(root::root).post(root::edit_userentries))
		.route("/entry/:id", get(entry::mark_read_redirect))
		.route("/feed/:id", get(feed::feed_page).post(feed::unsubscribe))
		.route("/user/:id", get(user::user_page))
		.route("/subscribe", get(subscribe::subscribe_page).post(subscribe::subscribe))
		.route("/error", get(|| async { error::HttpError::InternalError { description: "Juicy details!".to_string() }}))
		.route("/*any", any(|| async { error::HttpError::NotFound }))
		.layer(GlobalConcurrencyLimitLayer::with_semaphore(global_limit_semaphore))
		.layer(map_response(csp_header))
		.layer(CompressionLayer::new())
}

async fn csp_header<B>(mut response: Response<B>) -> Response<B> {
	response
		.headers_mut()
		.insert(
			"Content-Security-Policy",
			"script-source none".parse().unwrap(),
		);
	response
}

// Application state
#[derive(Debug)]
pub struct AppState<Persistence>
where Persistence: RussetPersistenceLayer {
	pub domain_service: Arc<RussetDomainService<Persistence>>,
}
impl <Persistence> Clone for AppState<Persistence>
where Persistence: RussetPersistenceLayer {
	fn clone(&self) -> Self { AppState { domain_service: self.domain_service.clone() } }
}

#[derive(Debug, Deserialize)]
struct PageQuery {
	page_num: Option<usize>,
	page_size: Option<usize>,
}

