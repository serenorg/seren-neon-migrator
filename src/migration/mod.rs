// ABOUTME: Migration utilities module
// ABOUTME: Handles schema introspection, dump/restore, and data migration

pub mod checksum;
pub mod dump;
pub mod restore;
pub mod schema;

pub use checksum::{compare_tables, compute_table_checksum, ChecksumResult};
pub use dump::{dump_data, dump_globals, dump_schema};
pub use restore::{restore_data, restore_globals, restore_schema};
pub use schema::{list_databases, list_tables, DatabaseInfo, TableInfo};
