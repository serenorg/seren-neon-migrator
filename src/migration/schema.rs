// ABOUTME: Schema introspection utilities for migration planning
// ABOUTME: Discovers databases, tables, and objects that need migration

use anyhow::{Context, Result};
use tokio_postgres::Client;

#[derive(Debug, Clone)]
pub struct DatabaseInfo {
    pub name: String,
    pub owner: String,
}

#[derive(Debug, Clone)]
pub struct TableInfo {
    pub schema: String,
    pub name: String,
    pub row_count_estimate: i64,
}

/// List all non-system databases in the cluster
pub async fn list_databases(client: &Client) -> Result<Vec<DatabaseInfo>> {
    let rows = client
        .query(
            "SELECT datname, pg_catalog.pg_get_userbyid(datdba) as owner
             FROM pg_catalog.pg_database
             WHERE datistemplate = false
               AND datname NOT IN ('postgres', 'template0', 'template1')
             ORDER BY datname",
            &[],
        )
        .await
        .context("Failed to list databases")?;

    let databases = rows
        .iter()
        .map(|row| DatabaseInfo {
            name: row.get(0),
            owner: row.get(1),
        })
        .collect();

    Ok(databases)
}

/// List all tables in the current database
pub async fn list_tables(client: &Client) -> Result<Vec<TableInfo>> {
    let rows = client
        .query(
            "SELECT
                pg_tables.schemaname,
                pg_tables.tablename,
                COALESCE(n_live_tup, 0) as row_count
             FROM pg_catalog.pg_tables
             LEFT JOIN pg_catalog.pg_stat_user_tables
                ON pg_tables.schemaname = pg_stat_user_tables.schemaname
                AND pg_tables.tablename = pg_stat_user_tables.relname
             WHERE pg_tables.schemaname NOT IN ('pg_catalog', 'information_schema')
             ORDER BY pg_tables.schemaname, pg_tables.tablename",
            &[],
        )
        .await
        .context("Failed to list tables")?;

    let tables = rows
        .iter()
        .map(|row| TableInfo {
            schema: row.get(0),
            name: row.get(1),
            row_count_estimate: row.get(2),
        })
        .collect();

    Ok(tables)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::postgres::connect;

    #[tokio::test]
    #[ignore]
    async fn test_list_databases() {
        let url = std::env::var("TEST_SOURCE_URL").unwrap();
        let client = connect(&url).await.unwrap();

        let databases = list_databases(&client).await.unwrap();

        // Should have at least the current database
        assert!(!databases.is_empty());
        println!("Found {} databases", databases.len());
        for db in &databases {
            println!("  - {} (owner: {})", db.name, db.owner);
        }
    }

    #[tokio::test]
    #[ignore]
    async fn test_list_tables() {
        let url = std::env::var("TEST_SOURCE_URL").unwrap();
        let client = connect(&url).await.unwrap();

        let tables = list_tables(&client).await.unwrap();

        // Result depends on test database, but should not error
        println!("Found {} tables", tables.len());
        for table in tables.iter().take(10) {
            println!(
                "  - {}.{} ({} rows)",
                table.schema, table.name, table.row_count_estimate
            );
        }
    }
}
