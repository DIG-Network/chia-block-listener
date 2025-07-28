use async_graphql::*;
use sqlx::{AnyPool, Row};
use std::sync::Arc;

use crate::database::DatabaseType;
use crate::resolvers::{
    CoreQueries, AddressQueries, BalanceQueries, CatQueries, NftQueries,
    NetworkQueries, PuzzleQueries, SolutionQueries, TemporalQueries,
    TransactionQueries, AnalyticsQueries,
};

/// Root query object that provides access to all blockchain data
pub struct QueryRoot {
    pool: Arc<AnyPool>,
    db_type: DatabaseType,
}

impl QueryRoot {
    pub fn new(pool: Arc<AnyPool>, db_type: DatabaseType) -> Self {
        Self { pool, db_type }
    }
}

#[Object]
impl QueryRoot {
    /// Core blockchain queries (blocks, coins, puzzles, solutions)
    async fn core(&self) -> CoreQueries {
        CoreQueries::new(self.pool.clone(), self.db_type)
    }

    /// Address-related queries
    async fn addresses(&self) -> AddressQueries {
        AddressQueries::new(self.pool.clone(), self.db_type)
    }

    /// Balance-related queries
    async fn balances(&self) -> BalanceQueries {
        BalanceQueries::new(self.pool.clone(), self.db_type)
    }

    /// CAT (Chia Asset Token) queries
    async fn cats(&self) -> CatQueries {
        CatQueries::new(self.pool.clone(), self.db_type)
    }

    /// NFT (Non-Fungible Token) queries
    async fn nfts(&self) -> NftQueries {
        NftQueries::new(self.pool.clone(), self.db_type)
    }

    /// Network statistics and analysis queries
    async fn network(&self) -> NetworkQueries {
        NetworkQueries::new(self.pool.clone(), self.db_type)
    }

    /// Puzzle-related queries
    async fn puzzles(&self) -> PuzzleQueries {
        PuzzleQueries::new(self.pool.clone(), self.db_type)
    }

    /// Solution-related queries
    async fn solutions(&self) -> SolutionQueries {
        SolutionQueries::new(self.pool.clone(), self.db_type)
    }

    /// Temporal analysis queries
    async fn temporal(&self) -> TemporalQueries {
        TemporalQueries::new(self.pool.clone(), self.db_type)
    }

    /// Transaction analysis queries
    async fn transactions(&self) -> TransactionQueries {
        TransactionQueries::new(self.pool.clone(), self.db_type)
    }

    /// Advanced analytics queries
    async fn analytics(&self) -> AnalyticsQueries {
        AnalyticsQueries::new(self.pool.clone(), self.db_type)
    }

    /// Direct query for a specific coin by ID
    async fn coin(&self, coin_id: String) -> Result<Option<Coin>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                "SELECT coin_id, parent_coin_info, puzzle_hash, amount, created_block 
                 FROM coins WHERE coin_id = $1"
            }
            DatabaseType::Sqlite => {
                "SELECT coin_id, parent_coin_info, puzzle_hash, amount, created_block 
                 FROM coins WHERE coin_id = ?"
            }
        };

        let row = sqlx::query(query)
            .bind(&coin_id)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| Coin {
            coin_id: r.get("coin_id"),
            parent_coin_info: hex::encode(r.get::<Vec<u8>, _>("parent_coin_info")),
            puzzle_hash: hex::encode(r.get::<Vec<u8>, _>("puzzle_hash")),
            amount: r.get::<i64, _>("amount") as u64,
            created_block: r.get("created_block"),
        }))
    }

    /// Direct query for a specific block by height
    async fn block(&self, height: i64) -> Result<Option<Block>> {
        let query = match self.db_type {
            DatabaseType::Postgres => {
                "SELECT height, weight, header_hash, timestamp 
                 FROM blocks WHERE height = $1"
            }
            DatabaseType::Sqlite => {
                "SELECT height, weight, header_hash, timestamp 
                 FROM blocks WHERE height = ?"
            }
        };

        let row = sqlx::query(query)
            .bind(height)
            .fetch_optional(&*self.pool)
            .await?;

        Ok(row.map(|r| Block {
            height: r.get("height"),
            weight: r.get("weight"),
            header_hash: hex::encode(r.get::<Vec<u8>, _>("header_hash")),
            timestamp: r.get("timestamp"),
        }))
    }

    /// Get the current blockchain height
    async fn current_height(&self) -> Result<i64> {
        let query = match self.db_type {
            DatabaseType::Postgres => "SELECT MAX(height) as max_height FROM blocks",
            DatabaseType::Sqlite => "SELECT MAX(height) as max_height FROM blocks",
        };

        let row = sqlx::query(query)
            .fetch_one(&*self.pool)
            .await?;

        Ok(row.get::<Option<i64>, _>("max_height").unwrap_or(0))
    }
}

/// Basic coin information
#[derive(Debug, Clone, SimpleObject)]
pub struct Coin {
    /// Unique coin ID
    pub coin_id: String,
    /// Parent coin info as hex
    pub parent_coin_info: String,
    /// Puzzle hash as hex
    pub puzzle_hash: String,
    /// Amount in mojos
    pub amount: u64,
    /// Block where created
    pub created_block: Option<i64>,
}

/// Basic block information
#[derive(Debug, Clone, SimpleObject)]
pub struct Block {
    /// Block height
    pub height: i64,
    /// Block weight
    pub weight: i64,
    /// Header hash as hex
    pub header_hash: String,
    /// Block timestamp
    pub timestamp: String,
} 