use crate::Result;
use crate::domain::RussetDomainService;
use crate::http::{ AppState, russet_router };
use crate::persistence::RussetPersistenceLayer;
use std::sync::Arc;
use std::time::Duration;
use tracing::{ error, info };

const SESSION_CLEANUP_INTERVAL: Duration = Duration::from_secs(3_600);

pub async fn start<Persistence>(
	domain_service: Arc<RussetDomainService<Persistence>>,
	listen: String,
	global_concurrent_limit: u32,
	login_concurrent_limit: u32,
) -> Result<()>
where Persistence: RussetPersistenceLayer {
	info!("Starting {}…", crate::APP_NAME);

	// Start the feed update coroutine
	let update_service = domain_service.clone();
	let check_interval = domain_service.feed_check_interval.clone();
	tokio::spawn(async move {
		loop {
			info!("Updating feeds");
			if let Err(errs) = update_service.update_feeds().await {
				for err in errs {
					error!(error = err);
				}
			}
			tokio::time::sleep(check_interval).await;
		}
	} );

	// Start the expired session cleanup coroutine
	let session_cleanup_service = domain_service.clone();
	tokio::spawn(async move {
		loop {
			info!("Removing expired sessions");
			if let Err(e) = session_cleanup_service.cleanup_expired_sessions().await {
				error!(error = e.as_ref(), "Error removing expired sessions");
			}
			tokio::time::sleep(SESSION_CLEANUP_INTERVAL).await;
		}
	} );

	// Start the HTTP server
	let app_state = AppState { domain_service: domain_service.clone() };
	let routes = russet_router(global_concurrent_limit, login_concurrent_limit)
		.with_state(app_state);
	let listener = tokio::net::TcpListener::bind(&listen).await?;
	let graceful_exit_signal = async {
		tokio::signal::ctrl_c().await.expect("Failed to register interrupt handler");
		info!("Received interrupt, exiting…");
	};
	info!("Initialization complete, serving requests!");
	info!("Listening on {listen}…");
	axum::serve(listener, routes)
		.with_graceful_shutdown(graceful_exit_signal)
		.await?;
	info!("Exiting {}…", crate::APP_NAME);
	Ok(())
}
