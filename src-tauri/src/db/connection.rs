/// Database connection management with connection pooling
///
/// Provides a thread-safe connection pool to SQLite database.

use crate::error::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use sqlx::ConnectOptions;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::Arc;

/// Maximum number of database connections in the pool
const MAX_CONNECTIONS: u32 = 5;

/// Database wrapper with connection pool
#[derive(Clone)]
pub struct Database {
    pool: Arc<SqlitePool>,
    db_path: PathBuf,
}

impl Database {
    /// Create a new database instance
    ///
    /// # Arguments
    /// * `db_path` - Path to the SQLite database file
    ///
    /// # Returns
    /// * `Ok(Database)` - Successfully created database instance
    /// * `Err(RecallError)` - If connection fails
    ///
    /// # Examples
    /// ```no_run
    /// use recall_cli_lib::db::Database;
    ///
    /// # async fn example() -> Result<(), Box<dyn std::error::Error>> {
    /// let db = Database::new("~/.recall/commands.db").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db_path = db_path.as_ref().to_path_buf();

        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        // Configure SQLite options
        let options = SqliteConnectOptions::from_str(&format!("sqlite:{}", db_path.display()))?
            .create_if_missing(true)
            .foreign_keys(true)
            .disable_statement_logging();

        // Create connection pool
        let pool = SqlitePoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .connect_with(options)
            .await?;

        let db = Self {
            pool: Arc::new(pool),
            db_path,
        };

        // Initialize schema
        db.initialize_schema().await?;

        Ok(db)
    }

    /// Create a test database in memory
    ///
    /// Used for testing. Creates a fresh database for each test.
    #[cfg(test)]
    pub async fn new_test() -> Result<Self> {
        let options = SqliteConnectOptions::from_str("sqlite::memory:")?
            .create_if_missing(true)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(MAX_CONNECTIONS)
            .connect_with(options)
            .await?;

        let db = Self {
            pool: Arc::new(pool),
            db_path: PathBuf::from(":memory:"),
        };

        db.initialize_schema().await?;

        Ok(db)
    }

    /// Initialize database schema
    ///
    /// Creates all required tables and indexes if they don't exist.
    async fn initialize_schema(&self) -> Result<()> {
        // Read schema file
        let schema = include_str!("../../../database/schema.sql");

        // Execute schema SQL
        // Note: SQLite doesn't support multiple statements in execute,
        // so we need to split and execute each statement
        for statement in schema.split(';') {
            let trimmed = statement.trim();
            if !trimmed.is_empty() {
                sqlx::query(trimmed).execute(self.pool.as_ref()).await?;
            }
        }

        Ok(())
    }

    /// Get reference to the connection pool
    ///
    /// Used internally by query modules.
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    /// Get the database file path
    pub fn path(&self) -> &Path {
        &self.db_path
    }

    /// Close all connections in the pool
    ///
    /// Should be called on application shutdown.
    pub async fn close(&self) {
        self.pool.close().await;
    }

    /// Get database statistics
    ///
    /// Returns information about the database for debugging.
    pub async fn stats(&self) -> Result<DatabaseStats> {
        let command_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM commands")
                .fetch_one(self.pool.as_ref())
                .await?;

        let pattern_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM command_patterns")
                .fetch_one(self.pool.as_ref())
                .await?;

        let suggestion_count: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM suggestions")
                .fetch_one(self.pool.as_ref())
                .await?;

        Ok(DatabaseStats {
            total_commands: command_count.0,
            total_patterns: pattern_count.0,
            total_suggestions: suggestion_count.0,
            pool_size: self.pool.size(),
            idle_connections: self.pool.num_idle(),
        })
    }
}

/// Database statistics
#[derive(Debug, Clone)]
pub struct DatabaseStats {
    pub total_commands: i64,
    pub total_patterns: i64,
    pub total_suggestions: i64,
    pub pool_size: u32,
    pub idle_connections: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_creation() {
        let db = Database::new_test().await;
        assert!(db.is_ok());
    }

    #[tokio::test]
    async fn test_database_stats() {
        let db = Database::new_test().await.unwrap();
        let stats = db.stats().await.unwrap();

        assert_eq!(stats.total_commands, 0);
        assert_eq!(stats.total_patterns, 0);
        assert_eq!(stats.total_suggestions, 0);
    }

    #[tokio::test]
    async fn test_database_pool() {
        let db = Database::new_test().await.unwrap();
        let pool = db.pool();

        assert_eq!(pool.size(), 1); // At least one connection in pool
    }

    #[tokio::test]
    async fn test_schema_initialization() {
        let db = Database::new_test().await.unwrap();

        // Verify tables exist by querying them
        let result: Result<(i64,), sqlx::Error> =
            sqlx::query_as("SELECT COUNT(*) FROM commands")
                .fetch_one(db.pool())
                .await;

        assert!(result.is_ok());
    }
}
