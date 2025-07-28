use crate::error::{PeerPoolError, Result};
use crate::types::*;
use chia_protocol::{FullBlock, Handshake as ChiaHandshake, NodeType, ProtocolMessageTypes, RequestBlock, RespondBlock};
use chia_traits::Streamable;
use futures_util::{SinkExt, StreamExt};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;
use tokio_tungstenite::{
    connect_async_tls_with_config, tungstenite::Message as WsMessage, 
    Connector, MaybeTlsStream, WebSocketStream
};
use tracing::{debug, error, info, warn};

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Represents a connection to a single Chia full node peer
#[derive(Clone)]
pub struct PeerConnection {
    pub peer_id: String,
    pub host: String,
    pub port: u16,
    pub network_id: String,
}

impl PeerConnection {
    /// Create a new peer connection
    pub fn new(peer_id: String, host: String, port: u16, network_id: String) -> Self {
        Self {
            peer_id,
            host,
            port,
            network_id,
        }
    }

    /// Connect to the peer and establish WebSocket connection
    pub async fn connect(&self, timeout_secs: u64) -> Result<WebSocket> {
        info!("Connecting to peer {} at {}:{}", self.peer_id, self.host, self.port);

        let connect_future = self.establish_connection();
        let ws_stream = timeout(Duration::from_secs(timeout_secs), connect_future)
            .await
            .map_err(|_| PeerPoolError::ConnectionTimeout)?
            .map_err(|e| PeerPoolError::Connection(e.to_string()))?;

        info!("Successfully connected to peer {}", self.peer_id);
        Ok(ws_stream)
    }

    /// Establish the actual WebSocket connection
    async fn establish_connection(&self) -> Result<WebSocket> {
        // Create TLS connector (simplified - in practice you'd use proper Chia certificates)
        let tls_connector = native_tls::TlsConnector::builder()
            .danger_accept_invalid_certs(true) // Only for development
            .build()
            .map_err(|e| PeerPoolError::Tls(e.to_string()))?;

        let connector = Connector::NativeTls(tls_connector);
        let ws_url = format!("wss://{}:{}/ws", self.host, self.port);

        let (ws_stream, _) = connect_async_tls_with_config(
            &ws_url,
            None,
            false,
            Some(connector),
        )
        .await
        .map_err(|e| PeerPoolError::WebSocket(e))?;

        Ok(ws_stream)
    }

    /// Perform Chia handshake with the peer
    pub async fn handshake(&self, ws_stream: &mut WebSocket) -> Result<()> {
        debug!("Starting handshake with peer {}", self.peer_id);

        // Create handshake message
        let handshake = ChiaHandshake {
            network_id: self.network_id.clone(),
            protocol_version: "0.0.36".to_string(),
            software_version: "chia-block-listener/1.0.0".to_string(),
            server_port: 8444,
            node_type: NodeType::FullNode as u8,
            capabilities: vec![
                (1, "base".to_string()),
                (2, "block_headers".to_string()),
                (3, "rate_limits_v2".to_string()),
            ],
        };

        // Serialize and send handshake
        let handshake_bytes = handshake
            .to_bytes()
            .map_err(|e| PeerPoolError::Protocol(format!("Failed to serialize handshake: {}", e)))?;

        ws_stream
            .send(WsMessage::Binary(handshake_bytes))
            .await
            .map_err(|e| PeerPoolError::WebSocket(e))?;

        // Wait for handshake response
        let response = timeout(Duration::from_secs(10), ws_stream.next())
            .await
            .map_err(|_| PeerPoolError::RequestTimeout)?
            .ok_or_else(|| PeerPoolError::Protocol("No handshake response".to_string()))?
            .map_err(|e| PeerPoolError::WebSocket(e))?;

        // Validate handshake response
        match response {
            WsMessage::Binary(data) => {
                // Try to parse as handshake response
                debug!("Received handshake response from peer {}", self.peer_id);
                // In practice, you'd validate the response properly
            }
            _ => {
                return Err(PeerPoolError::Protocol(
                    "Invalid handshake response format".to_string(),
                ));
            }
        }

        info!("Handshake completed with peer {}", self.peer_id);
        Ok(())
    }

    /// Request a block at specific height from the peer
    pub async fn request_block(&self, ws_stream: &mut WebSocket, height: u64) -> Result<FullBlock> {
        debug!("Requesting block {} from peer {}", height, self.peer_id);

        // Create block request
        let request = RequestBlock {
            height: height as u32,
            include_transaction_block: true,
        };

        // Serialize and send request
        let request_bytes = request
            .to_bytes()
            .map_err(|e| PeerPoolError::Protocol(format!("Failed to serialize request: {}", e)))?;

        // Create message with proper protocol type
        let message = self.create_protocol_message(ProtocolMessageTypes::RequestBlock, request_bytes)?;

        ws_stream
            .send(WsMessage::Binary(message))
            .await
            .map_err(|e| PeerPoolError::WebSocket(e))?;

        // Wait for response
        let response = timeout(Duration::from_secs(10), ws_stream.next())
            .await
            .map_err(|_| PeerPoolError::RequestTimeout)?
            .ok_or_else(|| PeerPoolError::Protocol("No block response".to_string()))?
            .map_err(|e| PeerPoolError::WebSocket(e))?;

        // Parse response
        match response {
            WsMessage::Binary(data) => {
                let (msg_type, payload) = self.parse_protocol_message(&data)?;
                
                match msg_type {
                    ProtocolMessageTypes::RespondBlock => {
                        let respond_block = RespondBlock::from_bytes(&payload)
                            .map_err(|e| PeerPoolError::Protocol(format!("Failed to parse response: {}", e)))?;
                        
                        if let Some(block) = respond_block.block {
                            debug!("Received block {} from peer {}", height, self.peer_id);
                            Ok(block)
                        } else {
                            Err(PeerPoolError::BlockNotFound { height })
                        }
                    }
                    _ => {
                        Err(PeerPoolError::Protocol(format!(
                            "Unexpected message type: {:?}",
                            msg_type
                        )))
                    }
                }
            }
            _ => {
                Err(PeerPoolError::Protocol(
                    "Invalid block response format".to_string(),
                ))
            }
        }
    }

    /// Create a properly formatted protocol message
    fn create_protocol_message(&self, msg_type: ProtocolMessageTypes, payload: Vec<u8>) -> Result<Vec<u8>> {
        let mut message = Vec::new();
        
        // Message type (1 byte)
        message.push(msg_type as u8);
        
        // Message ID (2 bytes) - using 0 for simplicity
        message.extend_from_slice(&[0u8, 0u8]);
        
        // Payload length (4 bytes)
        let payload_len = payload.len() as u32;
        message.extend_from_slice(&payload_len.to_be_bytes());
        
        // Payload
        message.extend_from_slice(&payload);
        
        Ok(message)
    }

    /// Parse a protocol message into type and payload
    fn parse_protocol_message(&self, data: &[u8]) -> Result<(ProtocolMessageTypes, Vec<u8>)> {
        if data.len() < 7 {
            return Err(PeerPoolError::Protocol("Message too short".to_string()));
        }

        let msg_type_byte = data[0];
        let msg_type = ProtocolMessageTypes::try_from(msg_type_byte)
            .map_err(|_| PeerPoolError::Protocol(format!("Unknown message type: {}", msg_type_byte)))?;

        let payload_len = u32::from_be_bytes([data[3], data[4], data[5], data[6]]) as usize;
        
        if data.len() < 7 + payload_len {
            return Err(PeerPoolError::Protocol("Incomplete message".to_string()));
        }

        let payload = data[7..7 + payload_len].to_vec();
        Ok((msg_type, payload))
    }

    /// Listen for blocks from the peer (for real-time sync)
    pub async fn listen_for_blocks(
        &self,
        mut ws_stream: WebSocket,
        block_sender: mpsc::Sender<BlockReceivedEvent>,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) -> Result<()> {
        info!("Starting block listener for peer {}", self.peer_id);

        loop {
            tokio::select! {
                message = ws_stream.next() => {
                    match message {
                        Some(Ok(WsMessage::Binary(data))) => {
                            if let Err(e) = self.handle_message(&data, &block_sender).await {
                                warn!("Error handling message from peer {}: {}", self.peer_id, e);
                            }
                        }
                        Some(Ok(WsMessage::Close(_))) => {
                            info!("Peer {} closed connection", self.peer_id);
                            break;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error from peer {}: {}", self.peer_id, e);
                            break;
                        }
                        None => {
                            info!("Connection to peer {} closed", self.peer_id);
                            break;
                        }
                        _ => {
                            // Ignore other message types
                        }
                    }
                }
                _ = &mut shutdown_rx => {
                    info!("Shutdown requested for peer {}", self.peer_id);
                    break;
                }
            }
        }

        // Clean up connection
        let _ = ws_stream.close(None).await;
        info!("Block listener stopped for peer {}", self.peer_id);
        Ok(())
    }

    /// Handle incoming messages from the peer
    async fn handle_message(
        &self,
        data: &[u8],
        block_sender: &mpsc::Sender<BlockReceivedEvent>,
    ) -> Result<()> {
        let (msg_type, payload) = self.parse_protocol_message(data)?;

        match msg_type {
            ProtocolMessageTypes::NewPeakWallet => {
                // Handle new peak notification
                debug!("Received new peak from peer {}", self.peer_id);
                // Parse and emit peak height event
            }
            ProtocolMessageTypes::RespondBlock => {
                // Handle unsolicited block
                let respond_block = RespondBlock::from_bytes(&payload)
                    .map_err(|e| PeerPoolError::Protocol(format!("Failed to parse block: {}", e)))?;
                
                if let Some(block) = respond_block.block {
                    self.emit_block_event(&block, block_sender).await?;
                }
            }
            _ => {
                // Ignore other message types
                debug!("Ignoring message type {:?} from peer {}", msg_type, self.peer_id);
            }
        }

        Ok(())
    }

    /// Convert a FullBlock to a BlockReceivedEvent and emit it
    async fn emit_block_event(
        &self,
        block: &FullBlock,
        block_sender: &mpsc::Sender<BlockReceivedEvent>,
    ) -> Result<()> {
        // Parse block using the generator parser
        let parser = chia_generator_parser::BlockParser::new();
        let parsed_block = parser.parse_block(block.clone())
            .map_err(|e| PeerPoolError::Protocol(format!("Failed to parse block: {}", e)))?;

        // Convert to event format
        let event = BlockReceivedEvent {
            height: parsed_block.height,
            header_hash: hex::encode(&parsed_block.header_hash),
            timestamp: parsed_block.timestamp.unwrap_or(0) as u64,
            weight: parsed_block.weight.to_string(),
            is_transaction_block: parsed_block.has_transactions_generator,
            peer_id: self.peer_id.clone(),
            received_at: chrono::Utc::now(),
            coin_additions: parsed_block.coin_additions.into_iter().map(|coin| CoinRecord {
                parent_coin_info: hex::encode(&coin.parent_coin_info),
                puzzle_hash: hex::encode(&coin.puzzle_hash),
                amount: coin.amount.to_string(),
            }).collect(),
            coin_removals: parsed_block.coin_removals.into_iter().map(|coin| CoinRecord {
                parent_coin_info: hex::encode(&coin.parent_coin_info),
                puzzle_hash: hex::encode(&coin.puzzle_hash),
                amount: coin.amount.to_string(),
            }).collect(),
            coin_spends: parsed_block.coin_spends.into_iter().map(|spend| CoinSpendRecord {
                coin: CoinRecord {
                    parent_coin_info: hex::encode(&spend.coin.parent_coin_info),
                    puzzle_hash: hex::encode(&spend.coin.puzzle_hash),
                    amount: spend.coin.amount.to_string(),
                },
                puzzle_reveal: hex::encode(&spend.puzzle_reveal),
                solution: hex::encode(&spend.solution),
            }).collect(),
            coin_creations: parsed_block.coin_creations.into_iter().map(|coin| CoinRecord {
                parent_coin_info: hex::encode(&coin.parent_coin_info),
                puzzle_hash: hex::encode(&coin.puzzle_hash),
                amount: coin.amount.to_string(),
            }).collect(),
        };

        // Send event
        block_sender.send(event).await
            .map_err(|_| PeerPoolError::Other("Failed to send block event".to_string()))?;

        Ok(())
    }

    /// Get connection info for this peer
    pub fn get_info(&self) -> PeerInfo {
        PeerInfo::new(self.peer_id.clone(), self.host.clone(), self.port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_peer_connection_creation() {
        let peer = PeerConnection::new(
            "test_peer".to_string(),
            "localhost".to_string(),
            8444,
            "mainnet".to_string(),
        );

        assert_eq!(peer.peer_id, "test_peer");
        assert_eq!(peer.host, "localhost");
        assert_eq!(peer.port, 8444);
        assert_eq!(peer.network_id, "mainnet");
    }

    #[test]
    fn test_protocol_message_creation() {
        let peer = PeerConnection::new(
            "test".to_string(),
            "localhost".to_string(),
            8444,
            "mainnet".to_string(),
        );

        let payload = vec![1, 2, 3, 4];
        let message = peer.create_protocol_message(ProtocolMessageTypes::RequestBlock, payload.clone()).unwrap();

        assert_eq!(message[0], ProtocolMessageTypes::RequestBlock as u8);
        assert_eq!(message[1..3], [0, 0]); // Message ID
        assert_eq!(message[3..7], [0, 0, 0, 4]); // Payload length
        assert_eq!(message[7..], payload); // Payload
    }

    #[test]
    fn test_protocol_message_parsing() {
        let peer = PeerConnection::new(
            "test".to_string(),
            "localhost".to_string(),
            8444,
            "mainnet".to_string(),
        );

        let payload = vec![1, 2, 3, 4];
        let message = peer.create_protocol_message(ProtocolMessageTypes::RequestBlock, payload.clone()).unwrap();
        let (msg_type, parsed_payload) = peer.parse_protocol_message(&message).unwrap();

        assert_eq!(msg_type, ProtocolMessageTypes::RequestBlock);
        assert_eq!(parsed_payload, payload);
    }
} 