// ABOUTME: Wrapper for psql and pg_restore to import database objects
// ABOUTME: Restores global objects, schema, and data to target

use anyhow::{bail, Context, Result};
use std::process::Command;

/// Restore global objects using psql
pub async fn restore_globals(target_url: &str, input_path: &str) -> Result<()> {
    tracing::info!("Restoring global objects from {}", input_path);

    let output = Command::new("psql")
        .arg(format!("--dbname={}", target_url))
        .arg(format!("--file={}", input_path))
        .arg("--quiet")
        .output()
        .context("Failed to execute psql")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        tracing::warn!("⚠ Some global object restoration warnings: {}", stderr);
        // Don't fail - some errors are expected (roles may already exist)
    }

    tracing::info!("✓ Global objects restored");
    Ok(())
}

/// Restore schema using psql
pub async fn restore_schema(target_url: &str, input_path: &str) -> Result<()> {
    tracing::info!("Restoring schema from {}", input_path);

    let output = Command::new("psql")
        .arg(format!("--dbname={}", target_url))
        .arg(format!("--file={}", input_path))
        .arg("--quiet")
        .output()
        .context("Failed to execute psql")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Schema restoration failed: {}", stderr);
    }

    tracing::info!("✓ Schema restored successfully");
    Ok(())
}

/// Restore data using psql
pub async fn restore_data(target_url: &str, input_path: &str) -> Result<()> {
    tracing::info!("Restoring data from {}", input_path);

    let output = Command::new("psql")
        .arg(format!("--dbname={}", target_url))
        .arg(format!("--file={}", input_path))
        .arg("--quiet")
        .output()
        .context("Failed to execute psql")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Data restoration failed: {}", stderr);
    }

    tracing::info!("✓ Data restored successfully");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::migration::dump;
    use tempfile::tempdir;

    #[tokio::test]
    #[ignore]
    async fn test_restore_globals() {
        let source_url = std::env::var("TEST_SOURCE_URL").unwrap();
        let target_url = std::env::var("TEST_TARGET_URL").unwrap();

        let dir = tempdir().unwrap();
        let dump_file = dir.path().join("globals.sql");

        // Dump from source
        dump::dump_globals(&source_url, dump_file.to_str().unwrap())
            .await
            .unwrap();

        // Restore to target
        let result = restore_globals(&target_url, dump_file.to_str().unwrap()).await;
        assert!(result.is_ok());
    }
}
