use axum::extract::State;
use axum::Router;
use axum::routing::{ any, get, post };
use axum::response::{ Html, Redirect };
use crate::domain::RussetDomainService;
use crate::persistence::RussetPersistenceLayer;
use crate::persistence::sql::SqlDatabase;
use std::sync::Arc;
use session::AuthenticatedUser;

mod session;
mod static_routes;
mod login;

pub fn russet_router() -> Router<AppState<SqlDatabase>> {
	Router::new()
		.route("/styles.css", get(static_routes::styles))
		.route("/login", get(login::login_page))
		.route("/login", post(login::login_user))
		.route("/whoami", get(whoami))
		.route("/", get(|| async { "User home page!" }))
		.route("/entry/:id", get(|| async { "Entry page!" }))
		.route("/*any", any(|| async { Redirect::to("/") }))
}

#[derive(Debug)]
pub struct AppState<Persistence>
where Persistence: RussetPersistenceLayer + Send + std::fmt::Debug {
	pub hello: String,
	pub domain_service: Arc<RussetDomainService<Persistence>>,
}
impl <Persistence> Clone for AppState<Persistence>
where Persistence: RussetPersistenceLayer + Send + std::fmt::Debug {
	fn clone(&self) -> Self { AppState { hello: self.hello.clone(), domain_service: self.domain_service.clone() } }
}

#[tracing::instrument]
async fn list_entries(
	State(state): State<AppState<SqlDatabase>>,
	user: AuthenticatedUser<SqlDatabase>,
) -> Html<String> {
	let feeds = state.domain_service.get_feeds().await;
	Html(format!("<pre>{:#?}</pre>", feeds))
}

#[tracing::instrument]
async fn whoami(
	State(state): State<AppState<SqlDatabase>>,
	AuthenticatedUser { user, .. }: AuthenticatedUser<SqlDatabase>,
) -> Html<String> {
	Html(format!("Authenticated as {}", user.name))
}

