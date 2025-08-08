mod domain;
mod application;
mod infrastructure;
mod http;

use std::net::SocketAddr;

use application::todo_service::TodoServiceImpl;
use domain::repository::TodoRepository;
use http::routing::{self, todos};
use infrastructure::sqlite_repo::SqliteTodoRepository;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let _ = dotenvy::dotenv();
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let database_url = std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://todos.db".to_string());
    // Ensure SQLite file can be created/opened when using a file-backed URL
    prepare_sqlite_file(&database_url)?;
    let repo = SqliteTodoRepository::connect(&database_url).await?;
    repo.init().await?;
    let service = TodoServiceImpl::new(repo);
    let todos_router = todos::router(todos::AppState { service });
    let router = routing::app(todos_router);

    let addr: SocketAddr = "127.0.0.1:3000".parse().unwrap();
    tracing::info!(%addr, "listening");
    axum::serve(tokio::net::TcpListener::bind(addr).await?, router)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

async fn shutdown_signal() {
    use tokio::signal::ctrl_c;
    let _ = ctrl_c().await;
    tracing::info!("shutdown");
}

fn prepare_sqlite_file(database_url: &str) -> anyhow::Result<()> {
    // Skip in-memory
    if database_url.starts_with("sqlite::memory:") { return Ok(()); }
    if let Some(path) = database_url.strip_prefix("sqlite://") {
        // On Windows, absolute paths may look like /C:/path; strip the leading slash
        let path = if cfg!(windows) && path.len() >= 3 && path.as_bytes()[0] == b'/' && path.as_bytes()[2] == b':' {
            &path[1..]
        } else {
            path
        };
        use std::{fs, path::Path, fs::OpenOptions};
        let p = Path::new(path);
        if let Some(parent) = p.parent() { if !parent.as_os_str().is_empty() { fs::create_dir_all(parent)?; } }
        if !p.exists() {
            let _ = OpenOptions::new().create(true).append(true).open(p)?;
        }
    }
    Ok(())
}
