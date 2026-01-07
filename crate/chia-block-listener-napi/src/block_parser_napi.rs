use chia_generator_parser::{
    parser::BlockParser as RustBlockParser,
    types::{BlockHeightInfo, CoinInfo, CoinSpendInfo, GeneratorBlockInfo, ParsedBlock},
};
use chia_protocol::FullBlock;
use chia_traits::streamable::Streamable;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use tracing::{debug, info};

// Export CoinInfo for TypeScript
#[napi(object)]
#[derive(Clone)]
pub struct CoinInfoJS {
    #[napi(js_name = "parentCoinInfo")]
    pub parent_coin_info: String,
    #[napi(js_name = "puzzleHash")]
    pub puzzle_hash: String,
    pub amount: String, // Use string for u64 to avoid JS precision issues
}

impl From<&CoinInfo> for CoinInfoJS {
    fn from(coin: &CoinInfo) -> Self {
        Self {
            parent_coin_info: coin.parent_coin_info.clone(),
            puzzle_hash: coin.puzzle_hash.clone(),
            amount: coin.amount.to_string(),
        }
    }
}

// Export CoinSpendInfo for TypeScript
#[napi(object)]
#[derive(Clone)]
pub struct CoinSpendInfoJS {
    pub coin: CoinInfoJS,
    #[napi(js_name = "puzzleReveal")]
    pub puzzle_reveal: String,
    pub solution: String,
    #[napi(js_name = "realData")]
    pub real_data: bool,
    #[napi(js_name = "parsingMethod")]
    pub parsing_method: String,
    pub offset: u32,
    #[napi(js_name = "createdCoins")]
    pub created_coins: Vec<CoinInfoJS>,
}

impl From<&CoinSpendInfo> for CoinSpendInfoJS {
    fn from(spend: &CoinSpendInfo) -> Self {
        Self {
            coin: (&spend.coin).into(),
            puzzle_reveal: spend.puzzle_reveal.clone(),
            solution: spend.solution.clone(),
            real_data: spend.real_data,
            parsing_method: spend.parsing_method.clone(),
            offset: spend.offset,
            created_coins: spend.created_coins.iter().map(|c| c.into()).collect(),
        }
    }
}

// Export ParsedBlock for TypeScript
#[napi(object)]
#[derive(Clone)]
pub struct ParsedBlockJS {
    pub height: u32,
    pub weight: String,
    #[napi(js_name = "headerHash")]
    pub header_hash: String,
    pub timestamp: Option<u32>,
    #[napi(js_name = "coinAdditions")]
    pub coin_additions: Vec<CoinInfoJS>,
    #[napi(js_name = "coinRemovals")]
    pub coin_removals: Vec<CoinInfoJS>,
    #[napi(js_name = "coinSpends")]
    pub coin_spends: Vec<CoinSpendInfoJS>,
    #[napi(js_name = "coinCreations")]
    pub coin_creations: Vec<CoinInfoJS>,
    #[napi(js_name = "hasTransactionsGenerator")]
    pub has_transactions_generator: bool,
    #[napi(js_name = "generatorSize")]
    pub generator_size: Option<u32>,
}

impl From<&ParsedBlock> for ParsedBlockJS {
    fn from(block: &ParsedBlock) -> Self {
        Self {
            height: block.height,
            weight: block.weight.clone(),
            header_hash: block.header_hash.clone(),
            timestamp: block.timestamp,
            coin_additions: block.coin_additions.iter().map(|c| c.into()).collect(),
            coin_removals: block.coin_removals.iter().map(|c| c.into()).collect(),
            coin_spends: block.coin_spends.iter().map(|s| s.into()).collect(),
            coin_creations: block.coin_creations.iter().map(|c| c.into()).collect(),
            has_transactions_generator: block.has_transactions_generator,
            generator_size: block.generator_size,
        }
    }
}

// Export GeneratorBlockInfo for TypeScript
#[napi(object)]
#[derive(Clone)]
pub struct GeneratorBlockInfoJS {
    #[napi(js_name = "prevHeaderHash")]
    pub prev_header_hash: String,
    #[napi(js_name = "transactionsGenerator")]
    pub transactions_generator: Option<String>, // Hex-encoded bytes
    #[napi(js_name = "transactionsGeneratorRefList")]
    pub transactions_generator_ref_list: Vec<u32>,
}

impl From<&GeneratorBlockInfo> for GeneratorBlockInfoJS {
    fn from(info: &GeneratorBlockInfo) -> Self {
        Self {
            prev_header_hash: hex::encode(info.prev_header_hash),
            transactions_generator: info.transactions_generator.as_ref().map(hex::encode),
            transactions_generator_ref_list: info.transactions_generator_ref_list.clone(),
        }
    }
}

// Export BlockHeightInfo for TypeScript
#[napi(object)]
#[derive(Clone)]
pub struct BlockHeightInfoJS {
    pub height: u32,
    #[napi(js_name = "isTransactionBlock")]
    pub is_transaction_block: bool,
}

impl From<&BlockHeightInfo> for BlockHeightInfoJS {
    fn from(info: &BlockHeightInfo) -> Self {
        Self {
            height: info.height,
            is_transaction_block: info.is_transaction_block,
        }
    }
}

#[napi]
pub struct ChiaBlockParser {
    parser: RustBlockParser,
}

impl Default for ChiaBlockParser {
    fn default() -> Self {
        Self::new()
    }
}

#[napi]
impl ChiaBlockParser {
    /// Create a new block parser
    #[napi(constructor)]
    pub fn new() -> Self {
        info!("Creating new ChiaBlockParser");
        Self {
            parser: RustBlockParser::new(),
        }
    }

    /// Parse a FullBlock from bytes
    #[napi]
    pub fn parse_full_block_from_bytes(&self, block_bytes: Buffer) -> Result<ParsedBlockJS> {
        debug!("Parsing FullBlock from {} bytes", block_bytes.len());

        let parsed_block = self
            .parser
            .parse_full_block_from_bytes(&block_bytes)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Parse error: {e}")))?;

        Ok((&parsed_block).into())
    }

    /// Parse a FullBlock from hex string
    #[napi]
    pub fn parse_full_block_from_hex(&self, block_hex: String) -> Result<ParsedBlockJS> {
        debug!(
            "Parsing FullBlock from hex string of length {}",
            block_hex.len()
        );

        let block_bytes = hex::decode(&block_hex)
            .map_err(|e| Error::new(Status::InvalidArg, format!("Hex decode error: {e}")))?;

        let parsed_block = self
            .parser
            .parse_full_block_from_bytes(&block_bytes)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Parse error: {e}")))?;

        Ok((&parsed_block).into())
    }

    /// Extract generator from block bytes
    #[napi]
    pub fn extract_generator_from_block_bytes(
        &self,
        block_bytes: Buffer,
    ) -> Result<Option<String>> {
        debug!("Extracting generator from {} bytes", block_bytes.len());

        let block = FullBlock::from_bytes(&block_bytes).map_err(|e| {
            Error::new(
                Status::InvalidArg,
                format!("Failed to deserialize FullBlock: {e}"),
            )
        })?;

        let generator = self
            .parser
            .extract_generator_from_block(&block)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Extract error: {e}")))?;

        Ok(generator.map(hex::encode))
    }

    /// Get block height and transaction status from block bytes
    #[napi]
    pub fn get_height_and_tx_status_from_block_bytes(
        &self,
        block_bytes: Buffer,
    ) -> Result<BlockHeightInfoJS> {
        debug!(
            "Getting height and tx status from {} bytes",
            block_bytes.len()
        );

        let block = FullBlock::from_bytes(&block_bytes).map_err(|e| {
            Error::new(
                Status::InvalidArg,
                format!("Failed to deserialize FullBlock: {e}"),
            )
        })?;

        let height_info = self
            .parser
            .get_height_and_tx_status_from_block(&block)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Parse error: {e}")))?;

        Ok((&height_info).into())
    }

    /// Parse block info from block bytes
    #[napi]
    pub fn parse_block_info_from_bytes(&self, block_bytes: Buffer) -> Result<GeneratorBlockInfoJS> {
        debug!("Parsing block info from {} bytes", block_bytes.len());

        let block = FullBlock::from_bytes(&block_bytes).map_err(|e| {
            Error::new(
                Status::InvalidArg,
                format!("Failed to deserialize FullBlock: {e}"),
            )
        })?;

        let block_info = self
            .parser
            .parse_block_info(&block)
            .map_err(|e| Error::new(Status::GenericFailure, format!("Parse error: {e}")))?;

        Ok((&block_info).into())
    }
}
