use chia_block_database::{ChiaBlockDatabase as InternalChiaBlockDatabase, DatabaseType as InternalDatabaseType, crud::blocks::Block as InternalBlock, crud::spends::Spend as InternalSpend};
use napi::bindgen_prelude::*;
use napi_derive::napi;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::info;
use chrono::{DateTime, Utc};
use serde_json;

// Export DatabaseType for TypeScript
#[napi]
#[derive(Debug)]
pub enum DatabaseType {
    Sqlite,
    Postgres,
}

impl From<DatabaseType> for InternalDatabaseType {
    fn from(db_type: DatabaseType) -> Self {
        match db_type {
            DatabaseType::Sqlite => InternalDatabaseType::Sqlite,
            DatabaseType::Postgres => InternalDatabaseType::Postgres,
        }
    }
}

impl From<InternalDatabaseType> for DatabaseType {
    fn from(db_type: InternalDatabaseType) -> Self {
        match db_type {
            InternalDatabaseType::Sqlite => DatabaseType::Sqlite,
            InternalDatabaseType::Postgres => DatabaseType::Postgres,
        }
    }
}

// Export Block for TypeScript
#[napi(object)]
pub struct Block {
    pub height: i64,
    pub weight: i64,
    #[napi(js_name = "headerHash")]
    pub header_hash: Vec<u8>,
    pub timestamp: String, // ISO 8601 string for compatibility
}

impl From<&InternalBlock> for Block {
    fn from(block: &InternalBlock) -> Self {
        Self {
            height: block.height,
            weight: block.weight,
            header_hash: block.header_hash.clone(),
            timestamp: block.timestamp.to_rfc3339(),
        }
    }
}

impl TryFrom<&Block> for InternalBlock {
    type Error = String;

    fn try_from(block: &Block) -> std::result::Result<Self, Self::Error> {
        let timestamp = DateTime::parse_from_rfc3339(&block.timestamp)
            .map_err(|e| format!("Invalid timestamp: {}", e))?
            .with_timezone(&Utc);

        Ok(Self {
            height: block.height,
            weight: block.weight,
            header_hash: block.header_hash.clone(),
            timestamp,
        })
    }
}

// Export Spend for TypeScript
#[napi(object)]
pub struct Spend {
    #[napi(js_name = "coinId")]
    pub coin_id: String,
    #[napi(js_name = "puzzleHash")]
    pub puzzle_hash: Option<Vec<u8>>,
    #[napi(js_name = "puzzleReveal")]
    pub puzzle_reveal: Option<Vec<u8>>,
    #[napi(js_name = "solutionHash")]
    pub solution_hash: Option<Vec<u8>>,
    pub solution: Option<Vec<u8>>,
    #[napi(js_name = "spentBlock")]
    pub spent_block: i64,
}

impl From<&InternalSpend> for Spend {
    fn from(spend: &InternalSpend) -> Self {
        Self {
            coin_id: spend.coin_id.clone(),
            puzzle_hash: spend.puzzle_hash.clone(),
            puzzle_reveal: spend.puzzle_reveal.clone(),
            solution_hash: spend.solution_hash.clone(),
            solution: spend.solution.clone(),
            spent_block: spend.spent_block,
        }
    }
}

impl From<&Spend> for InternalSpend {
    fn from(spend: &Spend) -> Self {
        Self {
            coin_id: spend.coin_id.clone(),
            puzzle_hash: spend.puzzle_hash.clone(),
            puzzle_reveal: spend.puzzle_reveal.clone(),
            solution_hash: spend.solution_hash.clone(),
            solution: spend.solution.clone(),
            spent_block: spend.spent_block,
        }
    }
}

// Blockchain Namespace
#[napi]
pub struct BlockchainNamespace {
    inner: Arc<RwLock<Option<InternalChiaBlockDatabase>>>,
}

#[napi]
impl BlockchainNamespace {
    // Block operations
    #[napi(js_name = "createBlock")]
    pub async fn create_block(&self, block: Block) -> Result<()> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let internal_block = InternalBlock::try_from(&block)
                .map_err(|e| Error::new(Status::InvalidArg, e))?;
            db.blockchain().blocks().create(db.pool(), &internal_block)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to create block: {}", e)))?;
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getBlockByHeight")]
    pub async fn get_block_by_height(&self, height: i64) -> Result<Option<Block>> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.blockchain().blocks().get_by_height(db.pool(), height)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get block: {}", e)))?;
            Ok(result.as_ref().map(|b| Block::from(b)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getBlocksRange")]
    pub async fn get_blocks_range(&self, start_height: i64, end_height: i64, page: i32, page_size: i32) -> Result<Vec<Block>> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.blockchain().blocks().get_range(db.pool(), start_height, end_height, page, page_size)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get blocks: {}", e)))?;
            Ok(result.iter().map(|b| Block::from(b)).collect())
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "deleteBlock")]
    pub async fn delete_block(&self, height: i64) -> Result<()> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            db.blockchain().blocks().delete(db.pool(), height)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to delete block: {}", e)))?;
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    // Spend operations
    #[napi(js_name = "createSpend")]
    pub async fn create_spend(&self, spend: Spend) -> Result<()> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let internal_spend = InternalSpend::from(&spend);
            db.blockchain().spends().create(db.pool(), &internal_spend)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to create spend: {}", e)))?;
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getSpendByCoinId")]
    pub async fn get_spend_by_coin_id(&self, coin_id: String) -> Result<Option<Spend>> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.blockchain().spends().get_by_coin_id(db.pool(), &coin_id)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get spend: {}", e)))?;
            Ok(result.as_ref().map(|s| Spend::from(s)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "deleteSpend")]
    pub async fn delete_spend(&self, coin_id: String) -> Result<()> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            db.blockchain().spends().delete(db.pool(), &coin_id)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to delete spend: {}", e)))?;
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }
}

// Analytics Namespace
#[napi]
pub struct AnalyticsNamespace {
    inner: Arc<RwLock<Option<InternalChiaBlockDatabase>>>,
}

#[napi]
impl AnalyticsNamespace {
    #[napi(js_name = "getPuzzleStats")]
    pub async fn get_puzzle_stats(&self) -> Result<String> {
        let _guard = self.inner.read().await;
        // Placeholder for puzzle analytics - would call the actual functions
        Ok("Puzzle analytics not yet implemented".to_string())
    }

    #[napi(js_name = "getSolutionStats")]
    pub async fn get_solution_stats(&self) -> Result<String> {
        let _guard = self.inner.read().await;
        // Placeholder for solution analytics - would call the actual functions
        Ok("Solution analytics not yet implemented".to_string())
    }

    #[napi(js_name = "getBalanceStats")]
    pub async fn get_balance_stats(&self) -> Result<String> {
        let _guard = self.inner.read().await;
        // Placeholder for balance analytics - would call the actual functions
        Ok("Balance analytics not yet implemented".to_string())
    }

    // Address Analysis Functions
    #[napi(js_name = "getAddressProfile")]
    pub async fn get_address_profile(&self, puzzle_hash_hex: String) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().address_analysis()
                .get_address_profile(db.pool(), &puzzle_hash_hex)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get address profile: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getAddressTransactionHistory")]
    pub async fn get_address_transaction_history(&self, puzzle_hash_hex: String, page: i32, page_size: i32) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().address_analysis()
                .get_address_transaction_history(db.pool(), &puzzle_hash_hex, page, page_size)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get transaction history: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getAddressCoinFlow")]
    pub async fn get_address_coin_flow(&self, puzzle_hash_hex: String) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().address_analysis()
                .get_address_coin_flow(db.pool(), &puzzle_hash_hex)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get coin flow: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getAddressSpendingBehavior")]
    pub async fn get_address_spending_behavior(&self, puzzle_hash_hex: String) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().address_analysis()
                .get_address_spending_behavior(db.pool(), &puzzle_hash_hex)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get spending behavior: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getAddressPeers")]
    pub async fn get_address_peers(&self, puzzle_hash_hex: String, page: i32, page_size: i32) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().address_analysis()
                .get_address_peers(db.pool(), &puzzle_hash_hex, page, page_size)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get address peers: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getAddressRiskAssessment")]
    pub async fn get_address_risk_assessment(&self, puzzle_hash_hex: String) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().address_analysis()
                .get_address_risk_assessment(db.pool(), &puzzle_hash_hex)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get risk assessment: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getAddressActivityTimeline")]
    pub async fn get_address_activity_timeline(&self, puzzle_hash_hex: String, days_back: i32) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().address_analysis()
                .get_address_activity_timeline(db.pool(), &puzzle_hash_hex, days_back)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get activity timeline: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    // CAT Analytics Functions
    #[napi(js_name = "getCatTokenAnalytics")]
    pub async fn get_cat_token_analytics(&self, asset_id: String) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().cat_analytics()
                .get_token_analytics(db.pool(), &asset_id)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get token analytics: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getCatDistributionAnalysis")]
    pub async fn get_cat_distribution_analysis(&self, asset_id: String, page: i32, page_size: i32) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().cat_analytics()
                .get_distribution_analysis(db.pool(), &asset_id, page, page_size)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get distribution analysis: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getCatTradingVelocity")]
    pub async fn get_cat_trading_velocity(&self, asset_id: String, days_back: i32) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().cat_analytics()
                .get_trading_velocity(db.pool(), &asset_id, days_back)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get trading velocity: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getCatHolderBehavior")]
    pub async fn get_cat_holder_behavior(&self, asset_id: String, page: i32, page_size: i32) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().cat_analytics()
                .get_holder_behavior(db.pool(), &asset_id, page, page_size)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get holder behavior: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getCatTokenComparison")]
    pub async fn get_cat_token_comparison(&self, page: i32, page_size: i32) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().cat_analytics()
                .get_token_comparison(db.pool(), page, page_size)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get token comparison: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getCatLiquidityAnalysis")]
    pub async fn get_cat_liquidity_analysis(&self, asset_id: String) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().cat_analytics()
                .get_liquidity_analysis(db.pool(), &asset_id)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get liquidity analysis: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    // NFT Analytics Functions
    #[napi(js_name = "getNftCollectionAnalytics")]
    pub async fn get_nft_collection_analytics(&self, collection_id: String) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().nft_analytics()
                .get_collection_analytics(db.pool(), &collection_id)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get collection analytics: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getNftOwnershipConcentration")]
    pub async fn get_nft_ownership_concentration(&self, collection_id: String, page: i32, page_size: i32) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().nft_analytics()
                .get_ownership_concentration(db.pool(), &collection_id, page, page_size)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get ownership concentration: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getNftTradingVelocity")]
    pub async fn get_nft_trading_velocity(&self, collection_id: String, days_back: i32) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().nft_analytics()
                .get_trading_velocity(db.pool(), &collection_id, days_back)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get trading velocity: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getNftRarityAnalysis")]
    pub async fn get_nft_rarity_analysis(&self, collection_id: String, page: i32, page_size: i32) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().nft_analytics()
                .get_rarity_analysis(db.pool(), &collection_id, page, page_size)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get rarity analysis: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getNftCollectionComparison")]
    pub async fn get_nft_collection_comparison(&self, page: i32, page_size: i32) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().nft_analytics()
                .get_collection_comparison(db.pool(), page, page_size)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get collection comparison: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    #[napi(js_name = "getNftCreatorAnalytics")]
    pub async fn get_nft_creator_analytics(&self, creator_puzzle_hash_hex: String) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            let result = db.analytics().nft_analytics()
                .get_creator_analytics(db.pool(), &creator_puzzle_hash_hex)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to get creator analytics: {}", e)))?;
            serde_json::to_string(&result)
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to serialize result: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }
}

// Assets Namespace
#[napi]
pub struct AssetsNamespace {
    inner: Arc<RwLock<Option<InternalChiaBlockDatabase>>>,
}

#[napi]
impl AssetsNamespace {
    #[napi(js_name = "getCatBalances")]
    pub async fn get_cat_balances(&self, owner_puzzle_hash: String, page: i32, page_size: i32) -> Result<String> {
        let _guard = self.inner.read().await;
        // Placeholder for CAT operations - would call the actual functions
        Ok(format!("CAT balances for {} (page {} of {})", owner_puzzle_hash, page, page_size))
    }

    #[napi(js_name = "getNftsByOwner")]
    pub async fn get_nfts_by_owner(&self, owner_puzzle_hash: String, collection_id: Option<String>, page: i32, page_size: i32) -> Result<String> {
        let _guard = self.inner.read().await;
        // Placeholder for NFT operations - would call the actual functions
        Ok(format!("NFTs for owner {} in collection {:?} (page {} of {})", owner_puzzle_hash, collection_id, page, page_size))
    }
}

// System Namespace
#[napi]
pub struct SystemNamespace {
    inner: Arc<RwLock<Option<InternalChiaBlockDatabase>>>,
}

#[napi]
impl SystemNamespace {
    #[napi(js_name = "getSyncStatus")]
    pub async fn get_sync_status(&self) -> Result<String> {
        let _guard = self.inner.read().await;
        // Placeholder for sync status - would call the actual functions
        Ok("Sync status not yet implemented".to_string())
    }

    #[napi(js_name = "getSystemPreferences")]
    pub async fn get_system_preferences(&self) -> Result<String> {
        let _guard = self.inner.read().await;
        // Placeholder for system preferences - would call the actual functions
        Ok("System preferences not yet implemented".to_string())
    }

    #[napi(js_name = "setSystemPreference")]
    pub async fn set_system_preference(&self, key: String, value: String) -> Result<()> {
        let _guard = self.inner.read().await;
        // Placeholder for setting system preferences
        info!("Setting system preference: {} = {}", key, value);
        Ok(())
    }
}

// Main ChiaBlockDatabase wrapper
#[napi]
pub struct ChiaBlockDatabase {
    inner: Arc<RwLock<Option<InternalChiaBlockDatabase>>>,
}

#[napi]
impl ChiaBlockDatabase {
    /// Create a new database instance
    #[napi(constructor)]
    pub fn new() -> Self {
        info!("Creating new ChiaBlockDatabase wrapper");
        Self {
            inner: Arc::new(RwLock::new(None)),
        }
    }

    /// Initialize the database with a connection URL
    #[napi]
    pub async fn init(&self, database_url: String) -> Result<()> {
        info!("Initializing ChiaBlockDatabase with URL: {}", database_url);

        let db = InternalChiaBlockDatabase::new(&database_url)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to create database: {}", e)))?;

        let mut guard = self.inner.write().await;
        *guard = Some(db);

        info!("ChiaBlockDatabase initialized successfully");
        Ok(())
    }

    /// Initialize the database with explicit type
    #[napi(js_name = "initWithType")]
    pub async fn init_with_type(&self, database_url: String, db_type: DatabaseType) -> Result<()> {
        info!("Initializing ChiaBlockDatabase with URL: {} and type: {:?}", database_url, db_type);

        let internal_type = InternalDatabaseType::from(db_type);
        let db = InternalChiaBlockDatabase::new_with_type(&database_url, internal_type)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to create database: {}", e)))?;

        let mut guard = self.inner.write().await;
        *guard = Some(db);

        info!("ChiaBlockDatabase initialized successfully with explicit type");
        Ok(())
    }

    /// Get the database type
    #[napi(js_name = "getDatabaseType")]
    pub async fn get_database_type(&self) -> Result<DatabaseType> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            Ok(DatabaseType::from(db.database_type().clone()))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    /// Test database connection
    #[napi(js_name = "testConnection")]
    pub async fn test_connection(&self) -> Result<bool> {
        let guard = self.inner.read().await;
        if let Some(_db) = guard.as_ref() {
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Access to blockchain operations (blocks, coins, spends)
    #[napi]
    pub fn blockchain(&self) -> BlockchainNamespace {
        BlockchainNamespace {
            inner: self.inner.clone(),
        }
    }

    /// Access to analytics operations (puzzles, solutions, balances)
    #[napi]
    pub fn analytics(&self) -> AnalyticsNamespace {
        AnalyticsNamespace {
            inner: self.inner.clone(),
        }
    }

    /// Access to asset operations (CATs, NFTs)
    #[napi]
    pub fn assets(&self) -> AssetsNamespace {
        AssetsNamespace {
            inner: self.inner.clone(),
        }
    }

    /// Access to system operations (sync status, preferences)
    #[napi]
    pub fn system(&self) -> SystemNamespace {
        SystemNamespace {
            inner: self.inner.clone(),
        }
    }

    /// Execute a raw SQL query with optional parameters
    #[napi(js_name = "executeRawQuery")]
    pub async fn execute_raw_query(&self, query: String, params: Option<Vec<String>>) -> Result<String> {
        let guard = self.inner.read().await;
        if let Some(db) = guard.as_ref() {
            // Convert string parameters to serde_json::Value
            let converted_params = if let Some(params) = params {
                let mut converted = Vec::new();
                for param in params {
                    // Try to parse as different types
                    let value = if let Ok(n) = param.parse::<i64>() {
                        serde_json::Value::Number(serde_json::Number::from(n))
                    } else if let Ok(f) = param.parse::<f64>() {
                        serde_json::Number::from_f64(f)
                            .map(serde_json::Value::Number)
                            .unwrap_or_else(|| serde_json::Value::String(param))
                    } else if let Ok(b) = param.parse::<bool>() {
                        serde_json::Value::Bool(b)
                    } else if param == "null" || param.is_empty() {
                        serde_json::Value::Null
                    } else {
                        serde_json::Value::String(param)
                    };
                    converted.push(value);
                }
                Some(converted)
            } else {
                None
            };

            db.execute_raw_query(&query, converted_params)
                .await
                .map_err(|e| Error::new(Status::GenericFailure, format!("Failed to execute raw query: {}", e)))
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }

    /// Close the database connection
    #[napi]
    pub async fn close(&self) -> Result<()> {
        let mut guard = self.inner.write().await;
        if let Some(db) = guard.take() {
            db.close().await;
            info!("ChiaBlockDatabase closed");
            Ok(())
        } else {
            Err(Error::new(Status::GenericFailure, "Database not initialized"))
        }
    }
} 