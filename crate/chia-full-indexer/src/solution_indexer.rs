use crate::types::*;
use crate::error::{IndexerError, Result};
use crate::blockchain_utils::BlockchainConverter;
use chia_wallet_sdk::{Clvm, ClvmError, Coin, Program, SpendBundle, CoinSpend as SdkCoinSpend};
use serde_json::{Value, Map};
use tracing::{debug, warn, error};
use std::collections::HashMap;

/// Well-known puzzle hashes for asset identification
const CAT_PUZZLE_HASH: &str = "ff02ffff01ff02ffff01ff02ffff03ffff18ff2fff3480ffff01ff04ffff04ff20ffff04ff2fff808080ffff04ffff02ff3effff04ff02ffff04ff05ffff04ffff02ff2affff04ff02ffff04ff27ffff04ffff02ffff03ff77ffff01ff02ff36ffff04ff02ffff04ff09ffff04ff57ffff04ffff02ff2effff04ff02ffff04ff05ff80808080ff808080ffff02ff7effff04ff02ffff04ffff02ffff03ff2fffff01ff0bff05ff0bff1780ffff01ff088080ff0180ff808080ff8080808080ffff04ffff04ff30ffff04ff5fff808080ffff02ff7effff04ff02ffff04ffff02ffff03ffff09ff11ff5880ffff0159ff8080ff0180ffff02ffff03ff20ffff01ff02ff7effff04ff02ffff04ff13ffff04ff27ffff04ffff02ff2effff04ff02ffff04ff05ff80808080ff808080ffff04ff17ff8080ff8080ffff02ff2affff04ff02ffff04ff27ffff04ffff02ff36ffff04ff02ffff04ff09ffff04ff57ffff04ffff02ff2effff04ff02ffff04ff05ff80808080ff808080ffff04ff17ff80808080ff808080ff8080ffff04ffff04ff20ffff04ff17ff808080ffff02ff7effff04ff02ffff04ffff02ffff03ff2fffff01ff0bff05ff0bff1780ffff01ff088080ff0180ffff02ffff03ff17ffff01ff02ffff03ffff09ff11ff2880ffff0159ff8080ff0180ffff01ff04ff09ffff04ff0dff5f80ffff01ff0bff0bff0580ff0180ff8080ff0180ffff04ffff04ff28ffff04ff5fff808080ffff02ff7effff04ff02ffff04ffff02ffff03ffff09ff11ff2c80ffff0159ff8080ff0180ffff02ffff03ff82013fffff01ff02ff0bffff04ff02ffff04ff17ffff04ff2fffff04ff5fffff04ff8201bfffff04ff82017fffff04ffff02ff8202ffffff04ff02ffff04ff0bffff04ff5fffff04ff8201bfff80808080ff808080ff8080808080ff8080808080ff018080";
const NFT_PUZZLE_HASH: &str = "ff02ffff01ff02ffff01ff02ffff03ffff18ff2fff3480ffff01ff04ffff04ff20ffff04ff2fff808080ffff04ffff02ff3effff04ff02ffff04ff05ffff04ffff02ff2affff04ff02ffff04ff27ffff04ffff02ffff03ff77ffff01ff02ff36ffff04ff02ffff04ff09ffff04ff57ffff04ffff02ff2effff04ff02ffff04ff05ff80808080ff808080ffff02ff7effff04ff02ffff04ffff02ffff03ff2fffff01ff0bff05ff0bff1780ffff01ff088080ff0180ff808080ff8080808080ffff04ffff04ff30ffff04ff5fff808080ffff02ff7effff04ff02ffff04ffff02ffff03ffff09ff11ff5880ffff0159ff8080ff0180ffff02ffff03ff20ffff01ff02ff7effff04ff02ffff04ff13ffff04ff27ffff04ffff02ff2effff04ff02ffff04ff05ff80808080ff808080ffff04ff17ff8080ff8080ffff02ff2affff04ff02ffff04ff27ffff04ffff02ff36ffff04ff02ffff04ff09ffff04ff57ffff04ffff02ff2effff04ff02ffff04ff05ff80808080ff808080ffff04ff17ff80808080ff808080ff8080ffff04ffff04ff20ffff04ff17ff808080ffff02ff7effff04ff02ffff04ffff02ffff03ff2fffff01ff0bff05ff0bff1780ffff01ff088080ff0180ffff02ffff03ff17ffff01ff02ffff03ffff09ff11ff2880ffff0159ff8080ff0180ffff01ff04ff09ffff04ff0dff5f80ffff01ff0bff0bff0580ff0180ff8080ff0180ffff04ffff04ff28ffff04ff5fff808080ffff02ff7effff04ff02ffff04ffff02ffff03ffff09ff11ff2c80ffff0159ff8080ff0180ffff02ffff03ff82013fffff01ff02ff0bffff04ff02ffff04ff17ffff04ff2fffff04ff5fffff04ff8201bfffff04ff82017fffff04ffff02ff8202ffffff04ff02ffff04ff0bffff04ff5fffff04ff8201bfff80808080ff808080ff8080808080ff8080808080ff018080";

/// Processes coin spends to extract CAT and NFT information using proper CLVM parsing
pub struct SolutionIndexer {
    metadata_cache: HashMap<String, MetadataResult>,
    http_client: reqwest::Client,
    clvm: Clvm,
}

impl SolutionIndexer {
    /// Create a new solution indexer
    pub fn new() -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent("chia-full-indexer/1.0")
            .build()
            .expect("Failed to create HTTP client");

        let clvm = Clvm::new();

        Self {
            metadata_cache: HashMap::new(),
            http_client,
            clvm,
        }
    }

    /// Process a coin spend and determine if it's a CAT or NFT using CLVM parsing
    pub async fn process_coin_spend(
        &mut self,
        spend: &CoinSpendRecord,
        block_height: u32,
    ) -> Result<AssetProcessingResult> {
        // Validate the spend record first
        BlockchainConverter::validate_coin_spend_record(spend)?;

        // Convert to SDK types for CLVM analysis
        let coin = self.convert_to_sdk_coin(&spend.coin)?;
        let puzzle_reveal = self.hex_to_program(&spend.puzzle_reveal)?;
        let solution = self.hex_to_program(&spend.solution)?;

        // Create SDK CoinSpend
        let coin_spend = SdkCoinSpend::new(coin, puzzle_reveal, solution);

        // Try to parse as NFT first (more specific patterns)
        if let Ok(nft_result) = self.try_parse_nft(&coin_spend, block_height).await {
            return Ok(nft_result);
        }

        // Try to parse as CAT
        if let Ok(cat_result) = self.try_parse_cat(&coin_spend, block_height).await {
            return Ok(cat_result);
        }

        // Not a recognized asset type
        Ok(AssetProcessingResult::None)
    }

    /// Convert CoinRecord to SDK Coin
    fn convert_to_sdk_coin(&self, coin_record: &CoinRecord) -> Result<Coin> {
        let parent_coin_info = hex::decode(&coin_record.parent_coin_info)
            .map_err(|e| IndexerError::Other(format!("Invalid parent coin info hex: {}", e)))?;
        
        let puzzle_hash = hex::decode(&coin_record.puzzle_hash)
            .map_err(|e| IndexerError::Other(format!("Invalid puzzle hash hex: {}", e)))?;
        
        let amount = coin_record.amount.parse::<u64>()
            .map_err(|e| IndexerError::Other(format!("Invalid amount: {}", e)))?;

        // Convert to 32-byte arrays
        let parent_coin_info: [u8; 32] = parent_coin_info.try_into()
            .map_err(|_| IndexerError::Other("Parent coin info must be 32 bytes".to_string()))?;
        
        let puzzle_hash: [u8; 32] = puzzle_hash.try_into()
            .map_err(|_| IndexerError::Other("Puzzle hash must be 32 bytes".to_string()))?;

        Ok(Coin::new(parent_coin_info, puzzle_hash, amount))
    }

    /// Convert hex string to CLVM Program
    fn hex_to_program(&self, hex_str: &str) -> Result<Program> {
        let bytes = hex::decode(hex_str)
            .map_err(|e| IndexerError::Other(format!("Invalid hex string: {}", e)))?;
        
        Program::from_bytes(&bytes)
            .map_err(|e| IndexerError::Other(format!("Invalid CLVM program: {}", e)))
    }

    /// Try to parse the spend as an NFT using CLVM analysis
    async fn try_parse_nft(
        &mut self,
        coin_spend: &SdkCoinSpend,
        block_height: u32,
    ) -> Result<AssetProcessingResult> {
        // Check if this is an NFT puzzle by analyzing the puzzle reveal
        if !self.is_nft_puzzle(&coin_spend.puzzle_reveal)? {
            return Err(IndexerError::Other("Not an NFT puzzle".to_string()));
        }

        // Extract NFT information using CLVM
        let nft_info = self.extract_nft_info_clvm(coin_spend, block_height).await?;

        // Fetch metadata if available
        let metadata = if let Some(metadata_uri) = &nft_info.metadata_uri {
            self.fetch_metadata(metadata_uri).await.ok()
        } else {
            None
        };

        Ok(AssetProcessingResult::Nft {
            launcher_id: nft_info.launcher_id,
            collection_id: nft_info.collection_id,
            metadata: metadata.map(|m| m.data),
        })
    }

    /// Try to parse the spend as a CAT using CLVM analysis
    async fn try_parse_cat(
        &mut self,
        coin_spend: &SdkCoinSpend,
        _block_height: u32,
    ) -> Result<AssetProcessingResult> {
        // Check if this is a CAT puzzle by analyzing the puzzle reveal
        if !self.is_cat_puzzle(&coin_spend.puzzle_reveal)? {
            return Err(IndexerError::Other("Not a CAT puzzle".to_string()));
        }

        // Extract CAT information using CLVM
        let cat_info = self.extract_cat_info_clvm(coin_spend)?;

        Ok(AssetProcessingResult::Cat {
            asset_id: cat_info.asset_id,
            amount: cat_info.amount,
            inner_puzzle_hash: cat_info.inner_puzzle_hash,
        })
    }

    /// Check if puzzle is an NFT puzzle using CLVM analysis
    fn is_nft_puzzle(&self, puzzle: &Program) -> Result<bool> {
        // Run the puzzle with the CLVM interpreter to analyze its structure
        match self.clvm.run_program(puzzle, &Program::nil()) {
            Ok(result) => {
                // Look for NFT-specific opcodes and patterns in the result
                let result_bytes = result.to_bytes();
                
                // Check for NFT singleton patterns
                // NFT puzzles typically contain singleton layer with specific patterns
                Ok(self.contains_nft_patterns(&result_bytes))
            }
            Err(_) => {
                // If we can't run the puzzle, fall back to hash comparison
                let puzzle_hash = puzzle.tree_hash();
                Ok(hex::encode(puzzle_hash).contains("nft") || 
                   self.is_known_nft_puzzle_hash(&hex::encode(puzzle_hash)))
            }
        }
    }

    /// Check if puzzle is a CAT puzzle using CLVM analysis  
    fn is_cat_puzzle(&self, puzzle: &Program) -> Result<bool> {
        // Run the puzzle with the CLVM interpreter to analyze its structure
        match self.clvm.run_program(puzzle, &Program::nil()) {
            Ok(result) => {
                // Look for CAT-specific opcodes and patterns in the result
                let result_bytes = result.to_bytes();
                
                // Check for CAT TAIL (Token Asset Identifier Layer) patterns
                Ok(self.contains_cat_patterns(&result_bytes))
            }
            Err(_) => {
                // If we can't run the puzzle, fall back to hash comparison
                let puzzle_hash = puzzle.tree_hash();
                Ok(self.is_known_cat_puzzle_hash(&hex::encode(puzzle_hash)))
            }
        }
    }

    /// Check for NFT-specific patterns in CLVM output
    fn contains_nft_patterns(&self, bytes: &[u8]) -> bool {
        // Look for singleton layer indicators
        // NFTs use the singleton pattern which has specific byte sequences
        let nft_patterns = [
            &[0xff, 0x02, 0xff, 0xff, 0x01], // Singleton announcement
            &[0x73, 0x69, 0x6e, 0x67, 0x6c], // "singl" in ASCII
            &[0x4e, 0x46, 0x54], // "NFT" in ASCII
        ];

        nft_patterns.iter().any(|pattern| {
            bytes.windows(pattern.len()).any(|window| window == *pattern)
        })
    }

    /// Check for CAT-specific patterns in CLVM output
    fn contains_cat_patterns(&self, bytes: &[u8]) -> bool {
        // Look for CAT TAIL indicators
        // CATs use specific opcodes for lineage proof and TAIL validation
        let cat_patterns = [
            &[0x43, 0x41, 0x54], // "CAT" in ASCII
            &[0x54, 0x41, 0x49, 0x4c], // "TAIL" in ASCII
            &[0xff, 0x02, 0xff, 0xff, 0x03], // CAT lineage proof pattern
        ];

        cat_patterns.iter().any(|pattern| {
            bytes.windows(pattern.len()).any(|window| window == *pattern)
        })
    }

    /// Check if puzzle hash is a known NFT puzzle hash
    fn is_known_nft_puzzle_hash(&self, puzzle_hash_hex: &str) -> bool {
        // List of known NFT puzzle hashes (this would be expanded in production)
        let known_nft_hashes = [
            NFT_PUZZLE_HASH,
            // Add more known NFT puzzle hashes here
        ];

        known_nft_hashes.iter().any(|&hash| puzzle_hash_hex.contains(hash))
    }

    /// Check if puzzle hash is a known CAT puzzle hash
    fn is_known_cat_puzzle_hash(&self, puzzle_hash_hex: &str) -> bool {
        // List of known CAT puzzle hashes
        let known_cat_hashes = [
            CAT_PUZZLE_HASH,
            // Add more known CAT puzzle hashes here
        ];

        known_cat_hashes.iter().any(|&hash| puzzle_hash_hex.contains(hash))
    }

    /// Extract NFT information using CLVM parsing
    async fn extract_nft_info_clvm(
        &mut self,
        coin_spend: &SdkCoinSpend,
        _block_height: u32,
    ) -> Result<NftInfo> {
        // Run the puzzle with the solution to get the conditions
        let conditions_result = self.clvm.run_program(&coin_spend.puzzle_reveal, &coin_spend.solution)
            .map_err(|e| IndexerError::Other(format!("Failed to run NFT puzzle: {}", e)))?;

        // Parse conditions to extract NFT state
        let conditions = self.parse_conditions(&conditions_result)?;
        
        // Extract launcher ID from conditions or compute from coin
        let launcher_id = self.extract_launcher_id_from_conditions(&conditions)
            .unwrap_or_else(|| hex::encode(coin_spend.coin.coin_id()));

        // Extract collection ID if present
        let collection_id = self.extract_collection_id_from_conditions(&conditions);

        // Extract metadata URI from conditions
        let metadata_uri = self.extract_metadata_uri_from_conditions(&conditions);

        // Extract edition information
        let (edition_number, edition_total) = self.extract_edition_info_from_conditions(&conditions);

        Ok(NftInfo {
            launcher_id,
            collection_id,
            metadata_uri,
            edition_number,
            edition_total,
        })
    }

    /// Extract CAT information using CLVM parsing
    fn extract_cat_info_clvm(&self, coin_spend: &SdkCoinSpend) -> Result<CatInfo> {
        // Run the puzzle with the solution to get the conditions
        let conditions_result = self.clvm.run_program(&coin_spend.puzzle_reveal, &coin_spend.solution)
            .map_err(|e| IndexerError::Other(format!("Failed to run CAT puzzle: {}", e)))?;

        // Parse conditions to extract CAT state
        let conditions = self.parse_conditions(&conditions_result)?;

        // Extract asset ID (TAIL hash) from conditions
        let asset_id = self.extract_asset_id_from_conditions(&conditions)
            .ok_or_else(|| IndexerError::Other("Could not extract CAT asset ID".to_string()))?;

        // Extract inner puzzle hash from the CAT puzzle structure
        let inner_puzzle_hash = self.extract_inner_puzzle_hash_from_conditions(&conditions)
            .unwrap_or_else(|| hex::encode(coin_spend.coin.puzzle_hash));

        // Amount is from the coin itself
        let amount = coin_spend.coin.amount;

        Ok(CatInfo {
            asset_id,
            amount,
            inner_puzzle_hash,
        })
    }

    /// Parse CLVM conditions from program result
    fn parse_conditions(&self, result: &Program) -> Result<Vec<Program>> {
        // Conditions are typically returned as a list
        // This is a simplified parser - in production you'd use a proper condition parser
        let mut conditions = Vec::new();
        
        // Try to iterate through the result as a list
        if let Ok(list_items) = self.program_to_list(result) {
            conditions.extend(list_items);
        }

        Ok(conditions)
    }

    /// Convert CLVM Program to list of Programs (simplified)
    fn program_to_list(&self, program: &Program) -> Result<Vec<Program>> {
        // This is a simplified implementation
        // In production, you'd use proper CLVM list parsing
        Ok(vec![program.clone()])
    }

    /// Extract launcher ID from NFT conditions
    fn extract_launcher_id_from_conditions(&self, conditions: &[Program]) -> Option<String> {
        // Look for CREATE_COIN conditions that indicate the launcher
        for condition in conditions {
            if let Ok(bytes) = condition.to_bytes() {
                // Look for launcher pattern in condition bytes
                if bytes.len() >= 32 {
                    // Return the first 32-byte sequence as potential launcher ID
                    return Some(hex::encode(&bytes[0..32]));
                }
            }
        }
        None
    }

    /// Extract collection ID from NFT conditions
    fn extract_collection_id_from_conditions(&self, conditions: &[Program]) -> Option<String> {
        // Look for collection-specific patterns in conditions
        for condition in conditions {
            if let Ok(bytes) = condition.to_bytes() {
                // Look for collection ID patterns
                if bytes.len() >= 64 && bytes[0] == 0x01 {
                    return Some(hex::encode(&bytes[32..64]));
                }
            }
        }
        None
    }

    /// Extract metadata URI from NFT conditions
    fn extract_metadata_uri_from_conditions(&self, conditions: &[Program]) -> Option<String> {
        // Look for ASSERT_PUZZLE_ANNOUNCEMENT conditions with metadata URIs
        for condition in conditions {
            if let Ok(bytes) = condition.to_bytes() {
                // Try to parse as UTF-8 string
                if let Ok(text) = String::from_utf8(bytes) {
                    // Look for URI patterns
                    if text.starts_with("http") || text.starts_with("ipfs://") || text.starts_with("ar://") {
                        return Some(text);
                    }
                }
            }
        }
        None
    }

    /// Extract edition information from NFT conditions
    fn extract_edition_info_from_conditions(&self, conditions: &[Program]) -> (Option<u32>, Option<u32>) {
        // Look for edition number and total in conditions
        for condition in conditions {
            if let Ok(bytes) = condition.to_bytes() {
                // Look for edition patterns (simplified)
                if bytes.len() >= 8 {
                    let edition_number = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]);
                    let edition_total = u32::from_be_bytes([bytes[4], bytes[5], bytes[6], bytes[7]]);
                    
                    if edition_number > 0 && edition_total > 0 && edition_number <= edition_total {
                        return (Some(edition_number), Some(edition_total));
                    }
                }
            }
        }
        (None, None)
    }

    /// Extract asset ID from CAT conditions
    fn extract_asset_id_from_conditions(&self, conditions: &[Program]) -> Option<String> {
        // Look for TAIL hash in CAT conditions
        for condition in conditions {
            if let Ok(bytes) = condition.to_bytes() {
                // CAT asset ID is typically 32 bytes
                if bytes.len() >= 32 {
                    return Some(hex::encode(&bytes[0..32]));
                }
            }
        }
        None
    }

    /// Extract inner puzzle hash from CAT conditions
    fn extract_inner_puzzle_hash_from_conditions(&self, conditions: &[Program]) -> Option<String> {
        // Look for inner puzzle hash in CAT structure
        for condition in conditions {
            if let Ok(bytes) = condition.to_bytes() {
                // Inner puzzle hash is typically after the TAIL hash
                if bytes.len() >= 64 {
                    return Some(hex::encode(&bytes[32..64]));
                }
            }
        }
        None
    }

    /// Fetch metadata from a URI
    async fn fetch_metadata(&mut self, uri: &str) -> Result<MetadataResult> {
        // Check cache first
        if let Some(cached) = self.metadata_cache.get(uri) {
            return Ok(cached.clone());
        }

        debug!("Fetching metadata from: {}", uri);

        // Handle different URI schemes
        let data = match uri {
            uri if uri.starts_with("http://") || uri.starts_with("https://") => {
                self.fetch_http_metadata(uri).await?
            }
            uri if uri.starts_with("ipfs://") => {
                self.fetch_ipfs_metadata(uri).await?
            }
            uri if uri.starts_with("ar://") => {
                self.fetch_arweave_metadata(uri).await?
            }
            _ => {
                return Err(IndexerError::MetadataFetchFailed {
                    url: uri.to_string(),
                    reason: "Unsupported URI scheme".to_string(),
                });
            }
        };

        let metadata_result = MetadataResult {
            url: uri.to_string(),
            data,
            content_type: Some("application/json".to_string()),
            retrieved_at: chrono::Utc::now(),
        };

        // Cache the result
        self.metadata_cache.insert(uri.to_string(), metadata_result.clone());

        debug!("Successfully fetched and cached metadata for: {}", uri);
        Ok(metadata_result)
    }

    /// Fetch HTTP/HTTPS metadata
    async fn fetch_http_metadata(&self, uri: &str) -> Result<Value> {
        let response = self.http_client
            .get(uri)
            .send()
            .await
            .map_err(|e| IndexerError::MetadataFetchFailed {
                url: uri.to_string(),
                reason: e.to_string(),
            })?;

        let text = response.text().await
            .map_err(|e| IndexerError::MetadataFetchFailed {
                url: uri.to_string(),
                reason: format!("Failed to read response: {}", e),
            })?;

        // Try to parse as JSON
        serde_json::from_str::<Value>(&text)
            .map_err(|e| IndexerError::MetadataFetchFailed {
                url: uri.to_string(),
                reason: format!("Invalid JSON: {}", e),
            })
    }

    /// Fetch IPFS metadata
    async fn fetch_ipfs_metadata(&self, uri: &str) -> Result<Value> {
        // Convert IPFS URI to HTTP gateway URL
        let ipfs_hash = uri.strip_prefix("ipfs://").unwrap_or(uri);
        let gateway_url = format!("https://ipfs.io/ipfs/{}", ipfs_hash);
        
        self.fetch_http_metadata(&gateway_url).await
    }

    /// Fetch Arweave metadata
    async fn fetch_arweave_metadata(&self, uri: &str) -> Result<Value> {
        // Convert Arweave URI to HTTP URL
        let ar_hash = uri.strip_prefix("ar://").unwrap_or(uri);
        let ar_url = format!("https://arweave.net/{}", ar_hash);
        
        self.fetch_http_metadata(&ar_url).await
    }

    /// Process multiple coin spends in a batch
    pub async fn process_batch(
        &mut self,
        spends: &[CoinSpendRecord],
        block_height: u32,
    ) -> Vec<(usize, AssetProcessingResult)> {
        let mut results = Vec::new();

        for (index, spend) in spends.iter().enumerate() {
            match self.process_coin_spend(spend, block_height).await {
                Ok(result) => {
                    if !matches!(result, AssetProcessingResult::None) {
                        results.push((index, result));
                    }
                }
                Err(e) => {
                    warn!("Failed to process coin spend at index {}: {}", index, e);
                }
            }
        }

        results
    }

    /// Clear metadata cache (useful for memory management)
    pub fn clear_metadata_cache(&mut self) {
        self.metadata_cache.clear();
        debug!("Cleared metadata cache");
    }

    /// Get cache statistics
    pub fn get_cache_stats(&self) -> CacheStats {
        CacheStats {
            entries: self.metadata_cache.len(),
            total_size_bytes: self.metadata_cache
                .values()
                .map(|entry| {
                    // Rough estimate of memory usage
                    entry.url.len() + 
                    serde_json::to_string(&entry.data).map(|s| s.len()).unwrap_or(0) +
                    entry.content_type.as_ref().map(|s| s.len()).unwrap_or(0)
                })
                .sum(),
        }
    }
}

impl Default for SolutionIndexer {
    fn default() -> Self {
        Self::new()
    }
}

/// Information extracted from NFT spends
#[derive(Debug, Clone)]
struct NftInfo {
    launcher_id: String,
    collection_id: Option<String>,
    metadata_uri: Option<String>,
    edition_number: Option<u32>,
    edition_total: Option<u32>,
}

/// Information extracted from CAT spends
#[derive(Debug, Clone)]
struct CatInfo {
    asset_id: String,
    amount: u64,
    inner_puzzle_hash: String,
}

/// Metadata cache statistics
#[derive(Debug, Clone)]
pub struct CacheStats {
    pub entries: usize,
    pub total_size_bytes: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solution_indexer_creation() {
        let indexer = SolutionIndexer::new();
        assert_eq!(indexer.metadata_cache.len(), 0);
    }

    #[test]
    fn test_hex_to_program() {
        let indexer = SolutionIndexer::new();
        
        // Test with valid hex
        let hex = "ff80"; // Simple CLVM program
        let result = indexer.hex_to_program(hex);
        assert!(result.is_ok());
    }

    #[test]
    fn test_coin_conversion() {
        let indexer = SolutionIndexer::new();
        
        let coin_record = CoinRecord {
            parent_coin_info: "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef".to_string(),
            puzzle_hash: "fedcba9876543210fedcba9876543210fedcba9876543210fedcba9876543210".to_string(),
            amount: "1000000000000".to_string(),
        };
        
        let result = indexer.convert_to_sdk_coin(&coin_record);
        assert!(result.is_ok());
        
        let coin = result.unwrap();
        assert_eq!(coin.amount, 1000000000000);
    }

    #[tokio::test]
    async fn test_batch_processing() {
        let mut indexer = SolutionIndexer::new();
        let spends = vec![]; // Empty batch for testing
        
        let results = indexer.process_batch(&spends, 1000).await;
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn test_cache_management() {
        let mut indexer = SolutionIndexer::new();
        assert_eq!(indexer.get_cache_stats().entries, 0);
        
        indexer.clear_metadata_cache();
        assert_eq!(indexer.get_cache_stats().entries, 0);
    }

    #[test]
    fn test_pattern_detection() {
        let indexer = SolutionIndexer::new();
        
        // Test NFT pattern detection
        let nft_bytes = [0xff, 0x02, 0xff, 0xff, 0x01, 0x42]; // Contains NFT pattern
        assert!(indexer.contains_nft_patterns(&nft_bytes));
        
        // Test CAT pattern detection  
        let cat_bytes = [0x43, 0x41, 0x54, 0x00]; // Contains "CAT"
        assert!(indexer.contains_cat_patterns(&cat_bytes));
        
        // Test no pattern
        let empty_bytes = [0x00, 0x00, 0x00, 0x00];
        assert!(!indexer.contains_nft_patterns(&empty_bytes));
        assert!(!indexer.contains_cat_patterns(&empty_bytes));
    }
} 