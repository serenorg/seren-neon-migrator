// ABOUTME: Publication management for logical replication on source database
// ABOUTME: Creates and manages PostgreSQL publications for table replication

use anyhow::{Context, Result};
use tokio_postgres::Client;

/// Create a publication for all tables
pub async fn create_publication(client: &Client, publication_name: &str) -> Result<()> {
    tracing::info!("Creating publication '{}'...", publication_name);

    let query = format!("CREATE PUBLICATION \"{}\" FOR ALL TABLES", publication_name);

    match client.execute(&query, &[]).await {
        Ok(_) => {
            tracing::info!("✓ Publication '{}' created successfully", publication_name);
            Ok(())
        }
        Err(e) => {
            let err_str = e.to_string();
            // Publication might already exist - that's okay
            if err_str.contains("already exists") {
                tracing::info!("✓ Publication '{}' already exists", publication_name);
                Ok(())
            } else {
                anyhow::bail!(
                    "Failed to create publication '{}': {}",
                    publication_name,
                    err_str
                )
            }
        }
    }
}

/// List all publications in the database
pub async fn list_publications(client: &Client) -> Result<Vec<String>> {
    let rows = client
        .query("SELECT pubname FROM pg_publication ORDER BY pubname", &[])
        .await
        .context("Failed to list publications")?;

    let publications: Vec<String> = rows.iter().map(|row| row.get(0)).collect();

    Ok(publications)
}

/// Drop a publication
pub async fn drop_publication(client: &Client, publication_name: &str) -> Result<()> {
    tracing::info!("Dropping publication '{}'...", publication_name);

    let query = format!("DROP PUBLICATION IF EXISTS \"{}\"", publication_name);

    client
        .execute(&query, &[])
        .await
        .context(format!("Failed to drop publication '{}'", publication_name))?;

    tracing::info!("✓ Publication '{}' dropped", publication_name);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postgres::connect;

    #[tokio::test]
    #[ignore]
    async fn test_create_and_list_publications() {
        let url = std::env::var("TEST_SOURCE_URL").unwrap();
        let client = connect(&url).await.unwrap();

        let pub_name = "test_publication";

        // Clean up if exists
        let _ = drop_publication(&client, pub_name).await;

        // Create publication
        let result = create_publication(&client, pub_name).await;
        match &result {
            Ok(_) => println!("✓ Publication created successfully"),
            Err(e) => {
                println!("Error creating publication: {:?}", e);
                // If Neon doesn't support publications, skip rest of test
                if e.to_string().contains("not supported") || e.to_string().contains("permission") {
                    println!("Skipping test - Neon might not support publications on pooler");
                    return;
                }
            }
        }
        assert!(result.is_ok(), "Failed to create publication");

        // List publications
        let pubs = list_publications(&client).await.unwrap();
        println!("Publications: {:?}", pubs);
        assert!(pubs.contains(&pub_name.to_string()));

        // Clean up
        drop_publication(&client, pub_name).await.unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn test_drop_publication() {
        let url = std::env::var("TEST_SOURCE_URL").unwrap();
        let client = connect(&url).await.unwrap();

        let pub_name = "test_drop_publication";

        // Create publication
        create_publication(&client, pub_name).await.unwrap();

        // Drop it
        let result = drop_publication(&client, pub_name).await;
        assert!(result.is_ok());

        // Verify it's gone
        let pubs = list_publications(&client).await.unwrap();
        assert!(!pubs.contains(&pub_name.to_string()));
    }
}
