use axum::extract::State;
use axum::http::StatusCode;
use axum::http::header;
use axum::response::Response;
use crate::http::AppState;
use crate::persistence::sql::SqlDatabase;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "static/styles.css.stpl")]
struct Css { }
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
