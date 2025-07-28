use async_graphql::*;
use sqlx::{AnyPool, Row};
use std::sync::Arc;

use crate::database::DatabaseType;
use crate::error::{GraphQLError, GraphQLResult};
use crate::schema::types::*;

/// Solution analytics queries
pub struct SolutionQueries {
    pool: Arc<AnyPool>,
    db_type: DatabaseType,
}

impl SolutionQueries {
    pub fn new(pool: Arc<AnyPool>, db_type: DatabaseType) -> Self {
        Self { pool, db_type }
    }
}

#[Object]
impl SolutionQueries {
    /// Get solution by hash
    async fn by_hash(&self, solution_hash_hex: String) -> GraphQLResult<Option<SolutionInfo>> {
        let solution_hash = hex::decode(&solution_hash_hex)
            .map_err(|_| GraphQLError::InvalidInput("Invalid hex string".to_string()))?;

        let query = match self.db_type {
            DatabaseType::Postgres => {
                "SELECT * FROM solutions WHERE solution_hash = $1"
            }
            DatabaseType::Sqlite => {
                "SELECT * FROM solutions WHERE solution_hash = ?"
            }
        };

        let row = sqlx::query(query)
            .bind(&solution_hash)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| SolutionInfo {
            id: r.get("id"),
            solution_hash: hex::encode(r.get::<Vec<u8>, _>("solution_hash")),
            solution: hex::encode(r.get::<Vec<u8>, _>("solution")),
            size: r.get("size"),
            first_used_block: r.get("first_used_block"),
        }))
    }

    /// Get solution statistics
    async fn stats(&self) -> GraphQLResult<SolutionStats> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                    COUNT(*) as total_solutions,
                    AVG(size) as avg_size,
                    MIN(size) as min_size,
                    MAX(size) as max_size,
                    COALESCE(SUM(size), 0) as total_bytes
                FROM solutions
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                    COUNT(*) as total_solutions,
                    AVG(size) as avg_size,
                    MIN(size) as min_size,
                    MAX(size) as max_size,
                    COALESCE(SUM(size), 0) as total_bytes
                FROM solutions
                "#
            }
        };

        let row = sqlx::query(query)
            .fetch_one(&*self.pool)
            .await?;

        Ok(SolutionStats {
            total_solutions: row.get("total_solutions"),
            avg_size: row.get("avg_size"),
            min_size: row.get("min_size"),
            max_size: row.get("max_size"),
            total_bytes: row.get("total_bytes"),
        })
    }

    /// Get most used solutions
    async fn most_used(
        &self,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<CommonSolution>> {
        let offset = (page - 1) * page_size;
        
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                    encode(sol.solution_hash, 'hex') as solution_hash_hex,
                    sol.size,
                    sol.first_used_block,
                    COUNT(DISTINCT s.coin_id) as usage_count
                FROM solutions sol
                LEFT JOIN spends s ON sol.id = s.solution_id
                GROUP BY sol.id, sol.solution_hash, sol.size, sol.first_used_block
                ORDER BY usage_count DESC, sol.size DESC
                LIMIT $1 OFFSET $2
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                    hex(sol.solution_hash) as solution_hash_hex,
                    sol.size,
                    sol.first_used_block,
                    COUNT(DISTINCT s.coin_id) as usage_count
                FROM solutions sol
                LEFT JOIN spends s ON sol.id = s.solution_id
                GROUP BY sol.id, sol.solution_hash, sol.size, sol.first_used_block
                ORDER BY usage_count DESC, sol.size DESC
                LIMIT ? OFFSET ?
                "#
            }
        };

        let rows = sqlx::query(query)
            .bind(page_size)
            .bind(offset)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(CommonSolution {
                solution_hash_hex: row.get("solution_hash_hex"),
                size: row.get("size"),
                first_used_block: row.get("first_used_block"),
                usage_count: row.get("usage_count"),
            });
        }

        Ok(results)
    }

    /// Get solution complexity analysis
    async fn complexity_analysis(&self) -> GraphQLResult<Vec<SolutionComplexityAnalysis>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH size_ranges AS (
                  SELECT 
                    CASE 
                      WHEN size <= 50 THEN 'tiny'
                      WHEN size <= 200 THEN 'small'
                      WHEN size <= 1000 THEN 'medium'
                      WHEN size <= 5000 THEN 'large'
                      ELSE 'huge'
                    END as complexity_category,
                    size,
                    COUNT(DISTINCT s.coin_id) as usage_count
                  FROM solutions sol
                  LEFT JOIN spends s ON sol.id = s.solution_id
                  GROUP BY sol.id, size
                )
                SELECT 
                  complexity_category,
                  COUNT(*) as solution_count,
                  ROUND(AVG(size), 2) as avg_size,
                  MIN(size) as min_size,
                  MAX(size) as max_size,
                  SUM(usage_count) as total_usage
                FROM size_ranges
                GROUP BY complexity_category
                ORDER BY avg_size
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                WITH size_ranges AS (
                  SELECT 
                    CASE 
                      WHEN size <= 50 THEN 'tiny'
                      WHEN size <= 200 THEN 'small'
                      WHEN size <= 1000 THEN 'medium'
                      WHEN size <= 5000 THEN 'large'
                      ELSE 'huge'
                    END as complexity_category,
                    size,
                    COUNT(DISTINCT s.coin_id) as usage_count
                  FROM solutions sol
                  LEFT JOIN spends s ON sol.id = s.solution_id
                  GROUP BY sol.id, size
                )
                SELECT 
                  complexity_category,
                  COUNT(*) as solution_count,
                  ROUND(AVG(size), 2) as avg_size,
                  MIN(size) as min_size,
                  MAX(size) as max_size,
                  SUM(usage_count) as total_usage
                FROM size_ranges
                GROUP BY complexity_category
                ORDER BY avg_size
                "#
            }
        };

        let rows = sqlx::query(query)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(SolutionComplexityAnalysis {
                complexity_category: row.get("complexity_category"),
                solution_count: row.get("solution_count"),
                avg_size: row.get("avg_size"),
                min_size: row.get("min_size"),
                max_size: row.get("max_size"),
                total_usage: row.get("total_usage"),
            });
        }

        Ok(results)
    }

    /// Get recent solutions
    async fn recent(
        &self,
        #[graphql(default = 100)] limit: i32,
    ) -> GraphQLResult<Vec<SolutionInfo>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                    id,
                    encode(solution_hash, 'hex') as solution_hash,
                    encode(solution, 'hex') as solution,
                    size,
                    first_used_block
                FROM solutions
                ORDER BY first_used_block DESC
                LIMIT $1
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                    id,
                    hex(solution_hash) as solution_hash,
                    hex(solution) as solution,
                    size,
                    first_used_block
                FROM solutions
                ORDER BY first_used_block DESC
                LIMIT ?
                "#
            }
        };

        let rows = sqlx::query(query)
            .bind(limit)
            .fetch_all(&*self.pool)
            .await?;

        let mut results = Vec::new();
        for row in rows {
            results.push(SolutionInfo {
                id: row.get("id"),
                solution_hash: row.get("solution_hash"),
                solution: row.get("solution"),
                size: row.get("size"),
                first_used_block: row.get("first_used_block"),
            });
        }

        Ok(results)
    }
} 