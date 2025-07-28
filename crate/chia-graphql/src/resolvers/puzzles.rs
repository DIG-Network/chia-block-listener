use async_graphql::*;
use sqlx::{AnyPool, Row};
use std::sync::Arc;

use crate::database::DatabaseType;
use crate::error::{GraphQLError, GraphQLResult};
use crate::schema::types::*;

/// Puzzle analytics queries
pub struct PuzzleQueries {
    pool: Arc<AnyPool>,
    db_type: DatabaseType,
}

impl PuzzleQueries {
    pub fn new(pool: Arc<AnyPool>, db_type: DatabaseType) -> Self {
        Self { pool, db_type }
    }
}

#[Object]
impl PuzzleQueries {
    /// Get puzzle by hash
    async fn by_hash(&self, puzzle_hash_hex: String) -> GraphQLResult<Option<PuzzleInfo>> {
        let puzzle_hash = hex::decode(&puzzle_hash_hex)
            .map_err(|_| GraphQLError::InvalidInput("Invalid hex string".to_string()))?;

        let query = match self.db_type {
            DatabaseType::Postgres => {
                "SELECT * FROM puzzles WHERE puzzle_hash = $1"
            }
            DatabaseType::Sqlite => {
                "SELECT * FROM puzzles WHERE puzzle_hash = ?"
            }
        };

        let row = sqlx::query(query)
            .bind(&puzzle_hash)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| PuzzleInfo {
            id: r.get("id"),
            puzzle_hash: hex::encode(r.get::<Vec<u8>, _>("puzzle_hash")),
            puzzle_reveal: r.get::<Option<Vec<u8>>, _>("puzzle_reveal")
                .map(|v| hex::encode(v)),
            reveal_size: r.get("reveal_size"),
            first_seen_block: r.get("first_seen_block"),
        }))
    }

    /// Get puzzle statistics
    async fn stats(&self) -> GraphQLResult<PuzzleStats> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                    COUNT(*) as total_puzzles,
                    COUNT(puzzle_reveal) as puzzles_with_reveals,
                    COUNT(*) - COUNT(puzzle_reveal) as puzzles_without_reveals,
                    AVG(reveal_size) as avg_reveal_size,
                    COALESCE(SUM(reveal_size), 0) as total_reveal_bytes
                FROM puzzles
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                    COUNT(*) as total_puzzles,
                    COUNT(puzzle_reveal) as puzzles_with_reveals,
                    COUNT(*) - COUNT(puzzle_reveal) as puzzles_without_reveals,
                    AVG(reveal_size) as avg_reveal_size,
                    COALESCE(SUM(reveal_size), 0) as total_reveal_bytes
                FROM puzzles
                "#
            }
        };

        let row = sqlx::query(query)
            .fetch_one(&*self.pool)
            .await?;

        Ok(PuzzleStats {
            total_puzzles: row.get("total_puzzles"),
            puzzles_with_reveals: row.get("puzzles_with_reveals"),
            puzzles_without_reveals: row.get("puzzles_without_reveals"),
            avg_reveal_size: row.get("avg_reveal_size"),
            total_reveal_bytes: row.get("total_reveal_bytes"),
        })
    }

    /// Get most used puzzles
    async fn most_used(
        &self,
        #[graphql(default = 1)] page: i32,
        #[graphql(default = 20)] page_size: i32,
    ) -> GraphQLResult<Vec<CommonPuzzle>> {
        let offset = (page - 1) * page_size;
        
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                SELECT 
                    encode(p.puzzle_hash, 'hex') as puzzle_hash_hex,
                    p.reveal_size,
                    p.first_seen_block,
                    COUNT(DISTINCT s.coin_id) as usage_count
                FROM puzzles p
                LEFT JOIN spends s ON p.id = s.puzzle_id
                WHERE p.puzzle_reveal IS NOT NULL
                GROUP BY p.id, p.puzzle_hash, p.reveal_size, p.first_seen_block
                ORDER BY usage_count DESC, p.reveal_size DESC
                LIMIT $1 OFFSET $2
                "#
            }
            DatabaseType::Sqlite => {
                r#"
                SELECT 
                    hex(p.puzzle_hash) as puzzle_hash_hex,
                    p.reveal_size,
                    p.first_seen_block,
                    COUNT(DISTINCT s.coin_id) as usage_count
                FROM puzzles p
                LEFT JOIN spends s ON p.id = s.puzzle_id
                WHERE p.puzzle_reveal IS NOT NULL
                GROUP BY p.id, p.puzzle_hash, p.reveal_size, p.first_seen_block
                ORDER BY usage_count DESC, p.reveal_size DESC
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
            results.push(CommonPuzzle {
                puzzle_hash_hex: row.get("puzzle_hash_hex"),
                reveal_size: row.get("reveal_size"),
                first_seen_block: row.get("first_seen_block"),
                usage_count: row.get("usage_count"),
            });
        }

        Ok(results)
    }

    /// Get puzzle complexity analysis
    async fn complexity_analysis(&self) -> GraphQLResult<Vec<PuzzleComplexityAnalysis>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                r#"
                WITH size_ranges AS (
                  SELECT 
                    CASE 
                      WHEN reveal_size <= 100 THEN 'tiny'
                      WHEN reveal_size <= 500 THEN 'small'
                      WHEN reveal_size <= 2000 THEN 'medium'
                      WHEN reveal_size <= 10000 THEN 'large'
                      ELSE 'huge'
                    END as complexity_category,
                    reveal_size,
                    COUNT(DISTINCT s.coin_id) as usage_count
                  FROM puzzles p
                  LEFT JOIN spends s ON p.id = s.puzzle_id
                  WHERE p.puzzle_reveal IS NOT NULL
                  GROUP BY p.id, reveal_size
                )
                SELECT 
                  complexity_category,
                  COUNT(*) as puzzle_count,
                  ROUND(AVG(reveal_size), 2) as avg_size,
                  MIN(reveal_size) as min_size,
                  MAX(reveal_size) as max_size,
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
                      WHEN reveal_size <= 100 THEN 'tiny'
                      WHEN reveal_size <= 500 THEN 'small'
                      WHEN reveal_size <= 2000 THEN 'medium'
                      WHEN reveal_size <= 10000 THEN 'large'
                      ELSE 'huge'
                    END as complexity_category,
                    reveal_size,
                    COUNT(DISTINCT s.coin_id) as usage_count
                  FROM puzzles p
                  LEFT JOIN spends s ON p.id = s.puzzle_id
                  WHERE p.puzzle_reveal IS NOT NULL
                  GROUP BY p.id, reveal_size
                )
                SELECT 
                  complexity_category,
                  COUNT(*) as puzzle_count,
                  ROUND(AVG(reveal_size), 2) as avg_size,
                  MIN(reveal_size) as min_size,
                  MAX(reveal_size) as max_size,
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
            results.push(PuzzleComplexityAnalysis {
                complexity_category: row.get("complexity_category"),
                puzzle_count: row.get("puzzle_count"),
                avg_size: row.get("avg_size"),
                min_size: row.get("min_size"),
                max_size: row.get("max_size"),
                total_usage: row.get("total_usage"),
            });
        }

        Ok(results)
    }
} 