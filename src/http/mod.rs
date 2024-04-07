use axum::extract::{ Form, State };
use axum::Router;
use axum::routing::{ get, post };
use axum::response::Html;
use crate::domain::RussetDomainService;
use crate::persistence::RussetPersistenceLayer;
use crate::persistence::sql::SqlDatabase;
use serde::Deserialize;
use std::sync::Arc;

pub fn russet_router() -> Router<AppState<SqlDatabase>> {
	Router::new()
		.route("/hello", get(hello))
		.route("/list", get(list_entries))
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
async fn list_entries(State(state): State<AppState<SqlDatabase>>) -> String {
	let feeds = state.domain_service.get_feeds().await;
	format!("{:?}", feeds)
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
#[tracing::instrument]
async fn login_user(
	State(state): State<AppState<SqlDatabase>>,
	Form(login): Form<LoginRequest>,
) -> String {
	state.domain_service.login_user(login.user_name, login.plaintext_password).await.unwrap().unwrap_or("lolno".to_string())
}
