use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::{ Html, Response };
use crate::http::AppState;
use crate::persistence::sql::SqlDatabase;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "static/login.stpl")]
struct LoginPage { }
#[tracing::instrument]
pub async fn login_page(
	State(_state): State<AppState<SqlDatabase>>,
) -> Html<String> {
	Html(LoginPage{}.render_once().unwrap())
}

#[derive(TemplateOnce)]
#[template(path = "static/styles.css.stpl")]
struct Css { }
// TODO: This is not working right for some reason (maybe Content-Type?)
#[tracing::instrument]
pub async fn styles(
	State(_state): State<AppState<SqlDatabase>>,
) -> Response<String> {
	Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "text/css")
		.body(Css{}.render_once().unwrap())
		.unwrap()
}
