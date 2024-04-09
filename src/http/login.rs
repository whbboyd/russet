use axum::extract::{ Form, State };
use axum_extra::extract::cookie::{ Cookie, CookieJar };
use axum::http::StatusCode;
use axum::response::{ Html, Redirect };
use crate::http::AppState;
use crate::persistence::sql::SqlDatabase;
use sailfish::TemplateOnce;
use serde::Deserialize;

#[derive(TemplateOnce)]
#[template(path = "static/login.stpl")]
struct LoginPage { }
#[tracing::instrument]
pub async fn login_page(
	State(_state): State<AppState<SqlDatabase>>,
) -> Html<String> {
	Html(LoginPage{}.render_once().unwrap())
}

#[derive(Deserialize, Clone)]
pub struct LoginRequest {
	user_name: String,
	plaintext_password: String,
	redirect_to: Option<String>,
}
impl std::fmt::Debug for LoginRequest {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("LoginRequest")
			.field("user_name", &self.user_name)
			.field("plaintext_password", &"<redacted>")
			.field("redirect_to", &self.redirect_to)
			.finish()
	}
}
#[axum_macros::debug_handler]
#[tracing::instrument]
pub async fn login_user(
	State(state): State<AppState<SqlDatabase>>,
	cookies: CookieJar,
	Form(login): Form<LoginRequest>,
) -> Result<(CookieJar, Redirect), StatusCode> {
	let session = state.domain_service.login_user(login.user_name, login.plaintext_password).await;
	match session {
		Ok(Some(session)) => Ok((
			cookies.add(Cookie::new("session_id", session.token)),
			Redirect::to(&login.redirect_to.unwrap_or("/".to_string())),
		)),
		Ok(None) => Err(StatusCode::UNAUTHORIZED),
		Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
	}
}
