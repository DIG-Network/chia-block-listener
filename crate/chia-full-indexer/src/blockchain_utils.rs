use crate::types::*;
use crate::error::{IndexerError, Result};
use chia_generator_parser::types::{CoinInfo, CoinSpendInfo, ParsedBlock};
use chia_block_database::crud::blocks::Block;
use chia_block_database::crud::spends::Spend;
use chrono::{DateTime, Utc};
use std::str::FromStr;

/// Utility functions for converting between different blockchain data formats
pub struct BlockchainConverter;

impl BlockchainConverter {
    /// Convert a ParsedBlock from the parser to a ProcessingBlock for the indexer
    pub fn convert_parsed_block(parsed_block: &ParsedBlock, peer_id: String) -> ProcessingBlock {
        ProcessingBlock {
            height: parsed_block.height,
            header_hash: hex::encode(&parsed_block.header_hash),
            timestamp: parsed_block.timestamp.unwrap_or(0) as u64,
            weight: parsed_block.weight.to_string(),
            is_transaction_block: parsed_block.has_transactions_generator,
            coin_additions: parsed_block.coin_additions.iter().map(Self::convert_coin_info).collect(),
            coin_removals: parsed_block.coin_removals.iter().map(Self::convert_coin_info).collect(),
            coin_spends: parsed_block.coin_spends.iter().map(Self::convert_coin_spend_info).collect(),
            coin_creations: parsed_block.coin_creations.iter().map(Self::convert_coin_info).collect(),
        }
    }

    /// Convert CoinInfo to CoinRecord
    pub fn convert_coin_info(coin_info: &CoinInfo) -> CoinRecord {
        CoinRecord {
            parent_coin_info: hex::encode(&coin_info.parent_coin_info),
            puzzle_hash: hex::encode(&coin_info.puzzle_hash),
            amount: coin_info.amount.to_string(),
        }
    }

    /// Convert CoinSpendInfo to CoinSpendRecord
    pub fn convert_coin_spend_info(spend_info: &CoinSpendInfo) -> CoinSpendRecord {
        CoinSpendRecord {
            coin: Self::convert_coin_info(&spend_info.coin),
            puzzle_reveal: hex::encode(&spend_info.puzzle_reveal),
            solution: hex::encode(&spend_info.solution),
            created_coins: spend_info.created_coins.iter().map(Self::convert_coin_info).collect(),
        }
    }

    /// Convert ProcessingBlock to database Block
    pub fn convert_to_db_block(processing_block: &ProcessingBlock) -> Result<Block> {
        let timestamp = DateTime::from_timestamp(processing_block.timestamp as i64, 0)
            .unwrap_or_else(|| Utc::now());

        Ok(Block {
            height: processing_block.height as i64,
            weight: processing_block.weight.parse::<i64>()
                .map_err(|e| IndexerError::Other(format!("Invalid weight: {}", e)))?,
            header_hash: hex::decode(&processing_block.header_hash)
                .map_err(|e| IndexerError::Other(format!("Invalid header hash: {}", e)))?,
            timestamp,
        })
    }

    /// Convert CoinSpendRecord to database Spend
    pub fn convert_to_db_spend(
        spend_record: &CoinSpendRecord, 
        spent_block: u32
    ) -> Result<Spend> {
        let coin_id = Self::compute_coin_id(&spend_record.coin)?;
        
        Ok(Spend {
            coin_id,
            puzzle_hash: Some(hex::decode(&spend_record.coin.puzzle_hash)
                .map_err(|e| IndexerError::Other(format!("Invalid puzzle hash: {}", e)))?),
            puzzle_reveal: Some(hex::decode(&spend_record.puzzle_reveal)
                .map_err(|e| IndexerError::Other(format!("Invalid puzzle reveal: {}", e)))?),
            solution_hash: None, // Will be computed if needed
            solution: Some(hex::decode(&spend_record.solution)
                .map_err(|e| IndexerError::Other(format!("Invalid solution: {}", e)))?),
            spent_block: spent_block as i64,
        })
    }

    /// Compute coin ID from coin record
    pub fn compute_coin_id(coin: &CoinRecord) -> Result<String> {
        use sha2::{Sha256, Digest};
        
        let parent_coin_info = hex::decode(&coin.parent_coin_info)
            .map_err(|e| IndexerError::Other(format!("Invalid parent coin info: {}", e)))?;
        let puzzle_hash = hex::decode(&coin.puzzle_hash)
            .map_err(|e| IndexerError::Other(format!("Invalid puzzle hash: {}", e)))?;
        let amount = coin.amount.parse::<u64>()
            .map_err(|e| IndexerError::Other(format!("Invalid amount: {}", e)))?;

        let mut hasher = Sha256::new();
        hasher.update(&parent_coin_info);
        hasher.update(&puzzle_hash);
        hasher.update(&amount.to_be_bytes());
        
        Ok(hex::encode(hasher.finalize()))
    }

    /// Extract field value from raw data with fallback field names
    pub fn extract_field(data: &serde_json::Value, field_names: &[&str]) -> Option<String> {
        for field_name in field_names {
            if let Some(value) = data.get(field_name) {
                match value {
                    serde_json::Value::String(s) => return Some(s.clone()),
                    serde_json::Value::Number(n) => return Some(n.to_string()),
                    _ => continue,
                }
            }
        }
        None
    }

    /// Convert hex string to bytes, handling optional "0x" prefix
    pub fn hex_to_bytes(hex_string: &str) -> Result<Vec<u8>> {
        let clean_hex = hex_string.strip_prefix("0x").unwrap_or(hex_string);
        hex::decode(clean_hex)
            .map_err(|e| IndexerError::Other(format!("Invalid hex string '{}': {}", hex_string, e)))
    }

    /// Convert bytes to hex string with "0x" prefix
    pub fn bytes_to_hex(bytes: &[u8]) -> String {
        format!("0x{}", hex::encode(bytes))
    }

    /// Validate coin record data
    pub fn validate_coin_record(coin: &CoinRecord) -> Result<()> {
        // Validate parent coin info (32 bytes)
        let parent_bytes = Self::hex_to_bytes(&coin.parent_coin_info)?;
        if parent_bytes.len() != 32 {
            return Err(IndexerError::Other(format!(
                "Invalid parent coin info length: expected 32 bytes, got {}",
                parent_bytes.len()
            )));
        }

        // Validate puzzle hash (32 bytes)
        let puzzle_bytes = Self::hex_to_bytes(&coin.puzzle_hash)?;
        if puzzle_bytes.len() != 32 {
            return Err(IndexerError::Other(format!(
                "Invalid puzzle hash length: expected 32 bytes, got {}",
                puzzle_bytes.len()
            )));
        }

        // Validate amount
        coin.amount.parse::<u64>()
            .map_err(|e| IndexerError::Other(format!("Invalid amount '{}': {}", coin.amount, e)))?;

        Ok(())
    }

    /// Validate coin spend record data
    pub fn validate_coin_spend_record(spend: &CoinSpendRecord) -> Result<()> {
        // Validate the coin
        Self::validate_coin_record(&spend.coin)?;

        // Validate puzzle reveal
        Self::hex_to_bytes(&spend.puzzle_reveal)?;

        // Validate solution
        Self::hex_to_bytes(&spend.solution)?;

        // Validate created coins
        for created_coin in &spend.created_coins {
            Self::validate_coin_record(created_coin)?;
        }

        Ok(())
    }

    /// Estimate block size in bytes (rough approximation)
    pub fn estimate_block_size(block: &ProcessingBlock) -> usize {
        let base_size = 200; // Block header and metadata
        let coin_size = 96; // Approximately 96 bytes per coin (32+32+8 + overhead)
        let spend_size = 1000; // Approximately 1KB per spend (puzzle reveal + solution)

        base_size
            + (block.coin_additions.len() * coin_size)
            + (block.coin_removals.len() * coin_size)
            + (block.coin_creations.len() * coin_size)
            + (block.coin_spends.len() * spend_size)
    }

    /// Create a summary of block processing results
    pub fn create_processing_summary(
        block: &ProcessingBlock,
        processing_time_ms: u64,
        cats_processed: usize,
        nfts_processed: usize,
    ) -> BlockProcessingResult {
        BlockProcessingResult {
            height: block.height,
            coins_added: block.coin_additions.len() + block.coin_creations.len(),
            coins_spent: block.coin_spends.len(),
            cats_processed,
            nfts_processed,
            processing_time_ms,
        }
    }

    /// Check if a puzzle hash represents a standard wallet
    pub fn is_standard_wallet_puzzle(puzzle_hash: &str) -> bool {
        // This is a simplified check - in reality, you'd need to check the actual puzzle
        // Standard wallet puzzles have specific patterns
        puzzle_hash.len() == 64 // 32 bytes hex encoded
    }

    /// Extract address from puzzle hash (simplified)
    pub fn puzzle_hash_to_address(puzzle_hash: &str, network_id: &str) -> Result<String> {
        // This is a simplified implementation
        // In practice, you'd use the proper Chia address encoding
        let prefix = match network_id {
            "mainnet" => "xch",
            "testnet11" => "txch",
            _ => return Err(IndexerError::Other(format!("Unknown network: {}", network_id))),
        };
        
        // Simplified address format (not real Chia address encoding)
        Ok(format!("{}_{}", prefix, &puzzle_hash[..16]))
    }
}

/// Utility functions for working with amounts and units
pub struct AmountUtils;

impl AmountUtils {
    const MOJOS_PER_XCH: u64 = 1_000_000_000_000; // 1 trillion mojos = 1 XCH

    /// Convert mojos to XCH
    pub fn mojos_to_xch(mojos: u64) -> f64 {
        mojos as f64 / Self::MOJOS_PER_XCH as f64
    }

    /// Convert XCH to mojos
    pub fn xch_to_mojos(xch: f64) -> u64 {
        (xch * Self::MOJOS_PER_XCH as f64) as u64
    }

    /// Format amount as human-readable string
    pub fn format_amount(mojos: u64) -> String {
        let xch = Self::mojos_to_xch(mojos);
        if xch >= 1.0 {
            format!("{:.6} XCH", xch)
        } else if mojos >= 1_000_000 {
            format!("{:.3} mXCH", xch * 1000.0)
        } else {
            format!("{} mojos", mojos)
        }
    }

    /// Parse amount string to mojos
    pub fn parse_amount(amount_str: &str) -> Result<u64> {
        if amount_str.ends_with(" XCH") {
            let xch_str = amount_str.strip_suffix(" XCH").unwrap();
            let xch = xch_str.parse::<f64>()
                .map_err(|e| IndexerError::Other(format!("Invalid XCH amount: {}", e)))?;
            Ok(Self::xch_to_mojos(xch))
        } else if amount_str.ends_with(" mojos") {
            let mojos_str = amount_str.strip_suffix(" mojos").unwrap();
            mojos_str.parse::<u64>()
                .map_err(|e| IndexerError::Other(format!("Invalid mojos amount: {}", e)))
        } else {
            // Assume it's already in mojos
            amount_str.parse::<u64>()
                .map_err(|e| IndexerError::Other(format!("Invalid amount: {}", e)))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_amount_conversion() {
        assert_eq!(AmountUtils::mojos_to_xch(1_000_000_000_000), 1.0);
        assert_eq!(AmountUtils::xch_to_mojos(1.0), 1_000_000_000_000);
        assert_eq!(AmountUtils::xch_to_mojos(0.5), 500_000_000_000);
    }

    #[test]
    fn test_amount_formatting() {
        assert_eq!(AmountUtils::format_amount(1_000_000_000_000), "1.000000 XCH");
        assert_eq!(AmountUtils::format_amount(500_000_000_000), "0.500000 XCH");
        assert_eq!(AmountUtils::format_amount(1_000_000), "1.000 mXCH");
        assert_eq!(AmountUtils::format_amount(1_000), "1000 mojos");
    }

    #[test]
    fn test_hex_conversion() {
        let bytes = vec![0x12, 0x34, 0x56, 0x78];
        let hex = BlockchainConverter::bytes_to_hex(&bytes);
        assert_eq!(hex, "0x12345678");
        
        let decoded = BlockchainConverter::hex_to_bytes(&hex).unwrap();
        assert_eq!(decoded, bytes);
    }

    #[test]
    fn test_coin_record_validation() {
        let valid_coin = CoinRecord {
            parent_coin_info: "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef".to_string(),
            puzzle_hash: "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321".to_string(),
            amount: "1000000000000".to_string(),
        };
        
        assert!(BlockchainConverter::validate_coin_record(&valid_coin).is_ok());
        
        let invalid_coin = CoinRecord {
            parent_coin_info: "0x1234".to_string(), // Too short
            puzzle_hash: "0xfedcba0987654321fedcba0987654321fedcba0987654321fedcba0987654321".to_string(),
            amount: "1000000000000".to_string(),
        };
        
        assert!(BlockchainConverter::validate_coin_record(&invalid_coin).is_err());
    }
} 