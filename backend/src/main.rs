//! Process entrypoint.
//!
//! Three modes, chosen by argument:
//!
//! * no arguments — open the pool, apply migrations, serve until signalled;
//! * `--migrate-only` — apply migrations and exit, for `make migrate`;
//! * `--health-check` — probe the running server and exit 0 or 1, so the
//!   container healthcheck needs no `curl` in the runtime image.

use std::net::SocketAddr;
use std::process::ExitCode;

use anyhow::Context;
use tokio::net::TcpListener;

use grocery_backend::config::{access_log_dir, Settings};
use grocery_backend::{build_app, db, logging};

/// How long the health probe waits before declaring the server unhealthy.
const HEALTH_CHECK_TIMEOUT_SECS: u64 = 5;

#[tokio::main]
async fn main() -> ExitCode {
    // Held until `main` returns: dropping it stops the access-log file writer.
    let _log_guard = logging::init(access_log_dir().as_deref());

    let mode = match Mode::from_args(std::env::args().skip(1)) {
        Ok(mode) => mode,
        Err(message) => {
            eprintln!("{message}\n\nusage: grocery-backend [--migrate-only | --health-check]");
            return ExitCode::FAILURE;
        }
    };

    match run(mode).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            // `{:#}` prints the whole `anyhow` context chain, which is what
            // makes a startup failure diagnosable from container logs.
            tracing::error!("{err:#}");
            ExitCode::FAILURE
        }
    }
}

/// What this invocation should do.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Serve,
    MigrateOnly,
    HealthCheck,
}

impl Mode {
    /// Parses the command line. Unknown arguments are an error rather than
    /// being ignored, so a typo in a Compose file is noticed immediately.
    fn from_args(args: impl Iterator<Item = String>) -> Result<Self, String> {
        let mut mode = Self::Serve;
        for arg in args {
            mode = match arg.as_str() {
                "--migrate-only" => Self::MigrateOnly,
                "--health-check" => Self::HealthCheck,
                other => return Err(format!("unknown argument: {other}")),
            };
        }
        Ok(mode)
    }
}

async fn run(mode: Mode) -> anyhow::Result<()> {
    let settings = Settings::from_env().context("reading configuration from the environment")?;

    match mode {
        Mode::HealthCheck => health_check(&settings).await,
        Mode::MigrateOnly => {
            let pool = db::connect(&settings)
                .await
                .context("connecting to PostgreSQL")?;
            db::migrate(&pool).await.context("applying migrations")?;
            tracing::info!("migrations up to date");
            Ok(())
        }
        Mode::Serve => serve(settings).await,
    }
}

/// Opens the pool, applies migrations, and serves until signalled.
async fn serve(settings: Settings) -> anyhow::Result<()> {
    let pool = db::connect(&settings)
        .await
        .context("connecting to PostgreSQL")?;
    db::migrate(&pool).await.context("applying migrations")?;

    let listener = TcpListener::bind(&settings.bind_address)
        .await
        .with_context(|| format!("binding {}", settings.bind_address))?;
    tracing::info!(address = %listener.local_addr()?, "backend listening");

    let app = build_app(pool, settings);
    axum::serve(
        listener,
        app.into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal())
    .await
    .context("serving HTTP")
}

/// Probes `/api/health` on the configured bind address.
///
/// Used as the container healthcheck, which is why it talks HTTP rather than
/// just opening a socket: a process that is listening but cannot answer is
/// not healthy.
async fn health_check(settings: &Settings) -> anyhow::Result<()> {
    // A 0.0.0.0 bind address is not a valid destination; probe loopback.
    let target = settings.bind_address.replace("0.0.0.0", "127.0.0.1");
    let url = format!("http://{target}/api/health");

    let response = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(HEALTH_CHECK_TIMEOUT_SECS))
        .build()?
        .get(&url)
        .send()
        .await
        .with_context(|| format!("probing {url}"))?;

    anyhow::ensure!(
        response.status().is_success(),
        "health endpoint answered {}",
        response.status()
    );
    Ok(())
}

/// Resolves on Ctrl-C or SIGTERM so Compose can stop the container cleanly.
async fn shutdown_signal() {
    let interrupt = async {
        if let Err(err) = tokio::signal::ctrl_c().await {
            tracing::error!(?err, "could not listen for Ctrl-C");
        }
    };

    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut stream) => {
                stream.recv().await;
            }
            Err(err) => tracing::error!(?err, "could not listen for SIGTERM"),
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = interrupt => {}
        _ = terminate => {}
    }
    tracing::info!("shutdown signal received");
}

#[cfg(test)]
mod tests {
    use super::Mode;

    fn parse(args: &[&str]) -> Result<Mode, String> {
        Mode::from_args(args.iter().map(|s| s.to_string()))
    }

    #[test]
    fn no_arguments_serves() {
        assert_eq!(parse(&[]).unwrap(), Mode::Serve);
    }

    #[test]
    fn recognises_each_mode() {
        assert_eq!(parse(&["--migrate-only"]).unwrap(), Mode::MigrateOnly);
        assert_eq!(parse(&["--health-check"]).unwrap(), Mode::HealthCheck);
    }

    #[test]
    fn an_unknown_argument_is_an_error_not_a_silent_serve() {
        assert!(parse(&["--migrate-onyl"]).is_err());
    }
}
