//! `noemad` binary: boots the NOEMA daemon on loopback.

use std::process::ExitCode;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let db = noema_core::daemon::data_dir().join("noema.db");
    match noema_core::daemon::run(db).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            tracing::error!(%e, "noemad failed");
            ExitCode::FAILURE
        }
    }
}
