use crate::Result;
use crate::domain::RussetDomainService;
use crate::http::{ AppState, russet_router };
use crate::persistence::RussetPersistenceLayer;
use std::sync::Arc;
use std::time::Duration;
use tracing::{ error, info };

pub async fn start<Persistence>(
	domain_service: Arc<RussetDomainService<Persistence>>,
	listen: String,
) -> Result<()>
where Persistence: RussetPersistenceLayer {
	info!("Starting {}…", crate::APP_NAME);
	// Start the feed update coroutine
	let update_service = domain_service.clone();
	tokio::spawn(async move {
		loop {
			info!("Updating feeds");
			if let Err(e) = update_service.update_feeds().await {
				error!(error = e.as_ref(), "Error updating feeds");
			}
			tokio::time::sleep(Duration::from_secs(/*FIXME*/3_600)).await;
		}
	} );

	// Setup for Axum
	let app_state = AppState { domain_service: domain_service.clone() };
	let routes = russet_router()
		.with_state(app_state);
	let listener = tokio::net::TcpListener::bind(&listen).await?;
	info!("Initialization complete, serving requests!");
	info!("Listening on {listen}…");
	axum::serve(listener, routes).await?;
	info!("Exiting {}…", crate::APP_NAME);
	Ok(())
}
