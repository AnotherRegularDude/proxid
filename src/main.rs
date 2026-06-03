use anyhow::Result;

use proxid::app;
use proxid::config;
use proxid::config::MergedData;

#[tokio::main]
async fn main() -> Result<()> {
    let settings = config::ConfigBuilder::new()
        .with_custom_config(std::env::var("PROXID_CONFIG").ok().map(MergedData::File))
        .load()?;
    proxid::init(&settings.logging)?;

    let state = app::build_state(settings)?;
    let addr = format!("{}:{}", state.settings().server.host, state.settings().server.port);

    tracing::info!("app state successfully initialized");

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    tracing::info!(addr = %listener.local_addr()?, "listening on");

    let router = app::build_router(state);

    axum::serve(listener, router)
        .with_graceful_shutdown(async {
            shutdown_signal().await;
        })
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = tokio::signal::ctrl_c();

    #[cfg(unix)]
    let sigterm = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Signal listener for SIGTERM properly initialized")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let sigterm = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => { tracing::info!("received SIGINT, shutting down"); }
        _ = sigterm => { tracing::info!("received SIGTERM, shutting down"); }
    }
}
