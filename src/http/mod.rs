use axum::extract::{ Form, State };
use axum::http::StatusCode;
use axum::Router;
use axum::routing::{ get, post };
use axum::response::{ Html, Redirect };
use axum_extra::extract::cookie::{ Cookie, CookieJar };
use crate::domain::RussetDomainService;
use crate::persistence::RussetPersistenceLayer;
use crate::persistence::sql::SqlDatabase;
use serde::Deserialize;
use std::sync::Arc;
use session::AuthenticatedUser;

mod session;

pub fn russet_router() -> Router<AppState<SqlDatabase>> {
	Router::new()
		.route("/list", get(list_entries))
		.route("/whoami", get(whoami))
		.route("/hello", get(hello))
		.route("/login", get(login_page))
		.route("/login", post(login_user))
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
async fn hello(State(state): State<AppState<SqlDatabase>>) -> String {
	state.hello
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
async fn login_page(
	State(_state): State<AppState<SqlDatabase>>,
) -> Html<&'static str> {
Html(r#"
<html>
	<head>
		<title>Russet login</title>
	</head>
	<body>
		<form action="/login" method="post">
			<div>
				<label for="user_name">User name: </label>
				<input type="text" name="user_name" />
			</div>
			<div>
				<label for="plaintext_password">Password: </label>
				<input type="password" name="plaintext_password" />
			</div>
			<div>
				<input type="submit" value = "Log in" />
			</div>
		</form>
	</body>
</html>
"#)
}

#[derive(Deserialize, Clone)]
struct LoginRequest {
	user_name: String,
	plaintext_password: String,
}
impl std::fmt::Debug for LoginRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("LoginRequest")
			.field("user_name", &self.user_name)
			.field("plaintext_password", &"<redacted>")
			.finish()
	}
}
#[axum_macros::debug_handler]
#[tracing::instrument]
async fn login_user(
	State(state): State<AppState<SqlDatabase>>,
	cookies: CookieJar,
	Form(login): Form<LoginRequest>,
) -> Result<(CookieJar, Redirect), StatusCode> {
	let session = state.domain_service.login_user(login.user_name, login.plaintext_password).await;
	match session {
		Ok(Some(session)) => Ok((
			cookies.add(Cookie::new("session_id", session.token)),
			Redirect::to("/whoami"),
		)),
		Ok(None) => Err(StatusCode::UNAUTHORIZED),
		Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
	}
}

#[tracing::instrument]
async fn whoami(
	State(state): State<AppState<SqlDatabase>>,
	AuthenticatedUser { user, .. }: AuthenticatedUser<SqlDatabase>,
) -> Html<String> {
	Html(format!("Authenticated as {}", user.name))
}
