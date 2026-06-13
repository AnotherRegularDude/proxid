pub mod app;
pub mod config;
pub mod core;

mod features;
mod infrastructure;

use anyhow::Result;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

pub use core::error::{AppError, AppResult};

pub fn init(config: &config::LoggingConfig) -> Result<()> {
    let default_filter = config.filter().parse()?;
    let filter = EnvFilter::try_from_default_env().unwrap_or(default_filter);

    tracing_subscriber::registry().with(filter).with(tracing_subscriber::fmt::layer()).init();

    Ok(())
}
