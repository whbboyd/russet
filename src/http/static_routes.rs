use axum::http::StatusCode;
use axum::http::header;
use axum::response::Response;
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "static/styles.css.stpl")]
struct Css { }
#[tracing::instrument]
pub async fn styles() -> Response<String> {
	// TODO: This isn't actually infallible. Do something vaguely reasonable if
	// it fails.
	Response::builder()
		.status(StatusCode::OK)
		.header(header::CONTENT_TYPE, "text/css")
		.body(Css{}.render_once().expect("rendering a static asset should work"))
		.expect("building a response with a static asset should work")
}
