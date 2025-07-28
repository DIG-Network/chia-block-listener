pub mod database;
pub mod error;
pub mod schema;
pub mod resolvers;

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use sqlx::{Any, AnyPool, Pool};
use std::sync::Arc;

pub use database::{DatabaseConfig, DatabaseType};
pub use error::{GraphQLError, GraphQLResult};
pub use schema::QueryRoot;

/// Main ChiaGraphql instance that provides GraphQL API for the blockchain database
pub struct ChiaGraphql {
    pool: Arc<AnyPool>,
    db_type: DatabaseType,
    schema: Schema<QueryRoot, EmptyMutation, EmptySubscription>,
}

impl ChiaGraphql {
    /// Create a new ChiaGraphql instance from a database configuration
    pub async fn from_config(config: DatabaseConfig) -> GraphQLResult<Self> {
        let (pool, db_type) = match config {
            DatabaseConfig::Postgres { connection_string } => {
                let pool = Pool::<Any>::connect(&connection_string).await?;
                (pool, DatabaseType::Postgres)
            }
            DatabaseConfig::Sqlite { file_path } => {
                let connection_string = format!("sqlite:{}", file_path);
                let pool = Pool::<Any>::connect(&connection_string).await?;
                (pool, DatabaseType::Sqlite)
            }
        };

        let pool: Arc<AnyPool> = Arc::new(pool);
        
        let schema = Schema::build(
            QueryRoot::new(pool.clone(), db_type.clone()),
            EmptyMutation,
            EmptySubscription,
        )
        .data(pool.clone())
        .data(db_type.clone())
        .finish();

        Ok(Self {
            pool,
            db_type,
            schema,
        })
    }

    /// Get the GraphQL schema
    pub fn schema(&self) -> &Schema<QueryRoot, EmptyMutation, EmptySubscription> {
        &self.schema
    }

    /// Get the database pool
    pub fn pool(&self) -> &AnyPool {
        &self.pool
    }

    /// Get the database type
    pub fn db_type(&self) -> &DatabaseType {
        &self.db_type
    }
} 