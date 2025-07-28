use chia_generator_parser::{
    parser::BlockParser,
    types::{BlockHeightInfo, CoinInfo, CoinSpendInfo, GeneratorBlockInfo, ParsedBlock},
};
use chia_protocol::FullBlock;
use chia_traits::streamable::Streamable;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use tracing::{debug, info};

// Export CoinInfo for TypeScript with clean name
#[napi(object, js_name = "CoinInfo")]
#[derive(Clone)]
pub struct CoinInfoWrapper {
    #[napi(js_name = "parentCoinInfo")]
    pub parent_coin_info: String,
    #[napi(js_name = "puzzleHash")]
    pub puzzle_hash: String,
    pub amount: String,
}

impl From<&CoinInfo> for CoinInfoWrapper {
    fn from(coin: &CoinInfo) -> Self {
        Self {
            parent_coin_info: hex::encode(&coin.parent_coin_info),
            puzzle_hash: hex::encode(&coin.puzzle_hash),
            amount: coin.amount.to_string(),
        }
    }
}

// Export CoinSpendInfo for TypeScript with clean name
#[napi(object, js_name = "CoinSpendInfo")]
#[derive(Clone)]
pub struct CoinSpendInfoWrapper {
    pub coin: CoinInfoWrapper,
    #[napi(js_name = "puzzleReveal")]
    pub puzzle_reveal: String,
    pub solution: String,
    #[napi(js_name = "realData")]
    pub real_data: bool,
    #[napi(js_name = "parsingMethod")]
    pub parsing_method: String,
    pub offset: u32,
    #[napi(js_name = "createdCoins")]
    pub created_coins: Vec<CoinInfoWrapper>,
}

impl From<&CoinSpendInfo> for CoinSpendInfoWrapper {
    fn from(spend: &CoinSpendInfo) -> Self {
        Self {
            coin: CoinInfoWrapper::from(&spend.coin),
            puzzle_reveal: hex::encode(&spend.puzzle_reveal),
            solution: hex::encode(&spend.solution),
            real_data: spend.real_data,
            parsing_method: spend.parsing_method.clone(),
            offset: spend.offset,
            created_coins: spend
                .created_coins
                .iter()
                .map(CoinInfoWrapper::from)
                .collect(),
        }
    }
}

// Export ParsedBlock for TypeScript with clean name
#[napi(object, js_name = "ParsedBlock")]
#[derive(Clone)]
pub struct ParsedBlockWrapper {
    pub height: u32,
    pub weight: String,
    #[napi(js_name = "headerHash")]
    pub header_hash: String,
    pub timestamp: Option<u32>,
    #[napi(js_name = "coinAdditions")]
    pub coin_additions: Vec<CoinInfoWrapper>,
    #[napi(js_name = "coinRemovals")]
    pub coin_removals: Vec<CoinInfoWrapper>,
    #[napi(js_name = "coinSpends")]
    pub coin_spends: Vec<CoinSpendInfoWrapper>,
    #[napi(js_name = "coinCreations")]
    pub coin_creations: Vec<CoinInfoWrapper>,
    #[napi(js_name = "hasTransactionsGenerator")]
    pub has_transactions_generator: bool,
    #[napi(js_name = "generatorSize")]
    pub generator_size: Option<u32>,
}

impl From<&ParsedBlock> for ParsedBlockWrapper {
    fn from(block: &ParsedBlock) -> Self {
        Self {
            height: block.height,
            weight: block.weight.to_string(),
            header_hash: hex::encode(&block.header_hash),
            timestamp: block.timestamp.map(|t| t as u32),
            coin_additions: block.coin_additions.iter().map(CoinInfoWrapper::from).collect(),
            coin_removals: block.coin_removals.iter().map(CoinInfoWrapper::from).collect(),
            coin_spends: block.coin_spends.iter().map(CoinSpendInfoWrapper::from).collect(),
            coin_creations: block.coin_creations.iter().map(CoinInfoWrapper::from).collect(),
            has_transactions_generator: block.has_transactions_generator,
            generator_size: block.generator_size,
        }
    }
}

// Export GeneratorBlockInfo for TypeScript with clean name
#[napi(object, js_name = "GeneratorBlockInfo")]
#[derive(Clone)]
pub struct GeneratorBlockInfoWrapper {
    #[napi(js_name = "prevHeaderHash")]
    pub prev_header_hash: String,
    #[napi(js_name = "transactionsGenerator")]
    pub transactions_generator: Option<String>,
    #[napi(js_name = "transactionsGeneratorRefList")]
    pub transactions_generator_ref_list: Vec<u32>,
}

impl From<&GeneratorBlockInfo> for GeneratorBlockInfoWrapper {
    fn from(info: &GeneratorBlockInfo) -> Self {
        Self {
            prev_header_hash: hex::encode(&info.prev_header_hash),
            transactions_generator: info
                .transactions_generator
                .as_ref()
                .map(|g| hex::encode(g)),
            transactions_generator_ref_list: info.transactions_generator_ref_list.clone(),
        }
    }
}

// Export BlockHeightInfo for TypeScript with clean name
#[napi(object, js_name = "BlockHeightInfo")]
#[derive(Clone)]
pub struct BlockHeightInfoWrapper {
    pub height: u32,
    #[napi(js_name = "isTransactionBlock")]
    pub is_transaction_block: bool,
}

impl From<&BlockHeightInfo> for BlockHeightInfoWrapper {
    fn from(info: &BlockHeightInfo) -> Self {
        Self {
            height: info.height,
            is_transaction_block: info.is_transaction_block,
        }
    }
}

#[napi]
pub struct ChiaBlockParser {
    parser: BlockParser,
}

#[napi]
impl ChiaBlockParser {
    #[napi(constructor)]
    pub fn new() -> Self {
        info!("Creating new ChiaBlockParser");
        Self {
            parser: BlockParser::new(),
        }
    }

    #[napi(js_name = "parseFullBlockFromBytes")]
    pub fn parse_full_block_from_bytes(&self, block_bytes: Buffer) -> Result<ParsedBlockWrapper> {
        debug!("Parsing full block from {} bytes", block_bytes.len());

        let block_data = block_bytes.as_ref();
        let parsed_block = self
            .parser
            .parse_full_block_from_bytes(block_data)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(ParsedBlockWrapper::from(&parsed_block))
    }

    #[napi(js_name = "parseFullBlockFromHex")]
    pub fn parse_full_block_from_hex(&self, block_hex: String) -> Result<ParsedBlockWrapper> {
        debug!("Parsing full block from hex: {}", &block_hex[..64.min(block_hex.len())]);

        let block_bytes = hex::decode(&block_hex)
            .map_err(|e| Error::new(Status::InvalidArg, format!("Invalid hex: {}", e)))?;

        let parsed_block = self
            .parser
            .parse_full_block_from_bytes(&block_bytes)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(ParsedBlockWrapper::from(&parsed_block))
    }

    #[napi(js_name = "extractGeneratorFromBlockBytes")]
    pub fn extract_generator_from_block_bytes(&self, block_bytes: Buffer) -> Result<Option<String>> {
        debug!("Extracting generator from {} bytes", block_bytes.len());

        let block_data = block_bytes.as_ref();
        let block = FullBlock::from_bytes(block_data).map_err(|e| {
            Error::new(
                Status::InvalidArg,
                format!("Failed to deserialize FullBlock: {}", e),
            )
        })?;

        let generator = self
            .parser
            .extract_generator_from_block(&block)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(generator.map(|g| hex::encode(g)))
    }

    #[napi(js_name = "getHeightAndTxStatusFromBlockBytes")]
    pub fn get_height_and_tx_status_from_block_bytes(
        &self,
        block_bytes: Buffer,
    ) -> Result<BlockHeightInfoWrapper> {
        debug!(
            "Getting height and transaction status from {} bytes",
            block_bytes.len()
        );

        let block_data = block_bytes.as_ref();
        let block = FullBlock::from_bytes(block_data).map_err(|e| {
            Error::new(
                Status::InvalidArg,
                format!("Failed to deserialize FullBlock: {}", e),
            )
        })?;

        let info = self
            .parser
            .get_height_and_tx_status_from_block(&block)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(BlockHeightInfoWrapper::from(&info))
    }

    #[napi(js_name = "parseBlockInfoFromBytes")]
    pub fn parse_block_info_from_bytes(&self, block_bytes: Buffer) -> Result<GeneratorBlockInfoWrapper> {
        debug!("Parsing block info from {} bytes", block_bytes.len());

        let block_data = block_bytes.as_ref();
        let block = FullBlock::from_bytes(block_data).map_err(|e| {
            Error::new(
                Status::InvalidArg,
                format!("Failed to deserialize FullBlock: {}", e),
            )
        })?;

        let info = self
            .parser
            .parse_block_info(&block)
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(GeneratorBlockInfoWrapper::from(&info))
    }
}

impl Default for ChiaBlockParser {
    fn default() -> Self {
        Self::new()
    }
}
