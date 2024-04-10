use axum::extract::State;
use axum::response::{ Html, Redirect };
use axum::Router;
use axum::routing::{ any, get };
use crate::domain::RussetDomainService;
use crate::http::session::AuthenticatedUser;
use crate::persistence::model::{ Entry, User };
use crate::persistence::RussetPersistenceLayer;
use crate::persistence::sql::SqlDatabase;
use sailfish::TemplateOnce;
use std::sync::Arc;

mod session;
mod static_routes;
mod login;

pub fn russet_router() -> Router<AppState<SqlDatabase>> {
	Router::new()
		.route("/styles.css", get(static_routes::styles))
		.route("/login", get(login::login_page).post(login::login_user))
		.route("/whoami", get(whoami))
		.route("/", get(home))
		.route("/entry/:id", get(|| async { "Entry page!" }))
		.route("/*any", any(|| async { Redirect::to("/") }))
}

#[derive(Debug)]
pub struct AppState<Persistence>
where Persistence: RussetPersistenceLayer + Send + std::fmt::Debug {
	pub domain_service: Arc<RussetDomainService<Persistence>>,
}
impl <Persistence> Clone for AppState<Persistence>
where Persistence: RussetPersistenceLayer + Send + std::fmt::Debug {
	fn clone(&self) -> Self { AppState { domain_service: self.domain_service.clone() } }
}

#[tracing::instrument]
async fn list_entries(
	State(state): State<AppState<SqlDatabase>>,
	user: AuthenticatedUser<SqlDatabase>,
) -> Html<String> {
	let feeds = state.domain_service.get_feeds().await;
	Html(format!("<pre>{:#?}</pre>", feeds))
}

#[derive(TemplateOnce)]
#[template(path = "home.stpl")]
pub struct HomePage<'a> {
	user: &'a User,
	entries: &'a [Entry],
}
#[axum_macros::debug_handler]
#[tracing::instrument]
async fn home(
	State(state): State<AppState<SqlDatabase>>,
	user: AuthenticatedUser<SqlDatabase>,
) -> Html<String> {
	let entries = state.domain_service
		.get_entries()
		.await
		.into_iter()
		.filter_map(|entry| entry.ok())
		.collect::<Vec<Entry>>();
	Html(HomePage{ user: &user.user, entries: entries.as_slice() }.render_once().unwrap())
}
	

#[tracing::instrument]
async fn whoami(
	State(state): State<AppState<SqlDatabase>>,
	AuthenticatedUser { user, .. }: AuthenticatedUser<SqlDatabase>,
) -> Html<String> {
	Html(format!("Authenticated as {}", user.name))
}

