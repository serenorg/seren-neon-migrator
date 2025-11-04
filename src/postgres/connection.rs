// ABOUTME: PostgreSQL connection utilities for Neon and Seren
// ABOUTME: Handles connection string parsing, TLS setup, and connection lifecycle

use anyhow::{Context, Result};
use tokio_postgres::Client;
use postgres_native_tls::MakeTlsConnector;
use native_tls::TlsConnector;

/// Connect to PostgreSQL database with TLS support
pub async fn connect(connection_string: &str) -> Result<Client> {
    // Parse connection string
    let _config = connection_string
        .parse::<tokio_postgres::Config>()
        .context("Invalid connection string format")?;

    // Set up TLS connector for cloud connections
    let tls_connector = TlsConnector::builder()
        .danger_accept_invalid_certs(false)
        .build()
        .context("Failed to build TLS connector")?;
    let tls = MakeTlsConnector::new(tls_connector);

    // Connect
    let (client, connection) = tokio_postgres::connect(connection_string, tls)
        .await
        .context("Failed to connect to database")?;

    // Spawn connection handler
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            tracing::error!("Connection error: {}", e);
        }
    });

    Ok(client)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_connect_with_invalid_url_returns_error() {
        let result = connect("invalid-url").await;
        assert!(result.is_err());
    }

    // NOTE: This test requires a real PostgreSQL instance
    // Skip if TEST_DATABASE_URL is not set
    #[tokio::test]
    #[ignore]
    async fn test_connect_with_valid_url_succeeds() {
        let url = std::env::var("TEST_DATABASE_URL")
            .expect("TEST_DATABASE_URL must be set for integration tests");

        let result = connect(&url).await;
        assert!(result.is_ok());
    }
}
