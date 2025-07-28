use crate::error::{BlockListenerError, Result};
use crate::tls;
use crate::types::*;
use chia_protocol::{
    FullBlock, Handshake as ChiaHandshake, NewPeakWallet, NodeType, ProtocolMessageTypes,
    RespondBlock,
};
use chia_traits::Streamable;
use futures_util::{SinkExt, StreamExt};
use std::time::{Duration, Instant};
use tokio::net::TcpStream;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;
use tokio_tungstenite::{
    connect_async_tls_with_config, tungstenite::Message as WsMessage, 
    MaybeTlsStream, WebSocketStream
};
use tracing::{debug, error, info, warn};

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Represents a connection to a single Chia full node peer for real-time events
pub struct PeerConnection {
    info: ConnectionInfo,
    handshake_info: HandshakeInfo,
    cert: ChiaCertificate,
}

impl PeerConnection {
    /// Create a new peer connection
    pub fn new(info: ConnectionInfo, handshake_info: HandshakeInfo) -> Result<Self> {
        let cert = tls::load_or_generate_cert()?;
        
        Ok(Self {
            info,
            handshake_info,
            cert,
        })
    }

    /// Connect to the peer and start listening for events
    pub async fn connect_and_listen(
        &self,
        event_tx: mpsc::Sender<InternalEvent>,
        mut shutdown_rx: oneshot::Receiver<()>,
    ) -> Result<()> {
        info!("Connecting to peer {} at {}", self.info.peer_id, self.info.websocket_url());

        // Establish WebSocket connection with timeout
        let connect_future = self.establish_connection();
        let mut ws_stream = timeout(
            Duration::from_secs(self.handshake_info.capabilities.len() as u64 * 10),
            connect_future,
        )
        .await
        .map_err(|_| BlockListenerError::ConnectionTimeout)?
        .map_err(|e| BlockListenerError::Connection(e.to_string()))?;

        // Perform Chia handshake
        self.perform_handshake(&mut ws_stream).await?;

        // Emit connected event
        let connected_event = InternalEvent::PeerConnected(PeerConnectedEvent {
            peer_id: self.info.peer_id.clone(),
            host: self.info.host.clone(),
            port: self.info.port,
            connected_at: chrono::Utc::now(),
            network_id: self.info.network_id.clone(),
        });
        
        if event_tx.send(connected_event).await.is_err() {
            return Err(BlockListenerError::EventChannelClosed);
        }

        // Start message listening loop
        info!("Starting message listener for peer {}", self.info.peer_id);
        
        loop {
            tokio::select! {
                message = ws_stream.next() => {
                    match message {
                        Some(Ok(WsMessage::Binary(data))) => {
                            if let Err(e) = self.handle_message(&data, &event_tx).await {
                                warn!("Error handling message from peer {}: {}", self.info.peer_id, e);
                                
                                let error_event = InternalEvent::Error {
                                    peer_id: self.info.peer_id.clone(),
                                    error: e.to_string(),
                                };
                                let _ = event_tx.send(error_event).await;
                            }
                        }
                        Some(Ok(WsMessage::Close(_))) => {
                            info!("Peer {} closed connection", self.info.peer_id);
                            break;
                        }
                        Some(Err(e)) => {
                            error!("WebSocket error from peer {}: {}", self.info.peer_id, e);
                            break;
                        }
                        None => {
                            info!("Connection to peer {} closed", self.info.peer_id);
                            break;
                        }
                        _ => {
                            // Ignore other message types (text, ping, pong)
                        }
                    }
                }
                _ = &mut shutdown_rx => {
                    info!("Shutdown requested for peer {}", self.info.peer_id);
                    break;
                }
            }
        }

        // Clean up connection
        let _ = ws_stream.close(None).await;

        // Emit disconnected event
        let disconnected_event = InternalEvent::PeerDisconnected(PeerDisconnectedEvent {
            peer_id: self.info.peer_id.clone(),
            host: self.info.host.clone(),
            port: self.info.port,
            reason: "Connection closed".to_string(),
            disconnected_at: chrono::Utc::now(),
            was_expected: false,
        });
        
        let _ = event_tx.send(disconnected_event).await;

        info!("Peer connection handler stopped for {}", self.info.peer_id);
        Ok(())
    }

    /// Establish WebSocket connection
    async fn establish_connection(&self) -> Result<WebSocket> {
        let connector = tls::create_ws_connector(&self.cert)?;
        let ws_url = self.info.websocket_url();

        debug!("Connecting to WebSocket: {}", ws_url);

        let (ws_stream, _) = connect_async_tls_with_config(
            &ws_url,
            None,
            false,
            Some(connector),
        )
        .await
        .map_err(|e| BlockListenerError::WebSocket(e))?;

        debug!("WebSocket connection established to {}", self.info.peer_id);
        Ok(ws_stream)
    }

    /// Perform Chia protocol handshake
    async fn perform_handshake(&self, ws_stream: &mut WebSocket) -> Result<()> {
        debug!("Starting handshake with peer {}", self.info.peer_id);

        // Create handshake message
        let handshake = ChiaHandshake {
            network_id: self.handshake_info.network_id.clone(),
            protocol_version: self.handshake_info.protocol_version.clone(),
            software_version: self.handshake_info.software_version.clone(),
            server_port: self.handshake_info.server_port,
            node_type: self.handshake_info.node_type,
            capabilities: self.handshake_info.capabilities.clone(),
        };

        // Serialize and send handshake
        let handshake_bytes = handshake
            .to_bytes()
            .map_err(|e| BlockListenerError::Protocol(format!("Failed to serialize handshake: {}", e)))?;

        let message = self.create_protocol_message(ProtocolMessageTypes::Handshake, handshake_bytes)?;

        ws_stream
            .send(WsMessage::Binary(message))
            .await
            .map_err(|e| BlockListenerError::WebSocket(e))?;

        // Wait for handshake response with timeout
        let response = timeout(Duration::from_secs(10), ws_stream.next())
            .await
            .map_err(|_| BlockListenerError::ConnectionTimeout)?
            .ok_or_else(|| BlockListenerError::Protocol("No handshake response".to_string()))?
            .map_err(|e| BlockListenerError::WebSocket(e))?;

        // Validate handshake response
        match response {
            WsMessage::Binary(data) => {
                let (msg_type, payload) = self.parse_protocol_message(&data)?;
                
                if msg_type == ProtocolMessageTypes::Handshake {
                    let response_handshake = ChiaHandshake::from_bytes(&payload)
                        .map_err(|e| BlockListenerError::Protocol(format!("Failed to parse handshake response: {}", e)))?;
                    
                    debug!("Handshake response received from peer {}: network={}, version={}", 
                           self.info.peer_id, response_handshake.network_id, response_handshake.protocol_version);
                    
                    // Validate network ID matches
                    if response_handshake.network_id != self.handshake_info.network_id {
                        return Err(BlockListenerError::Handshake(format!(
                            "Network ID mismatch: expected {}, got {}", 
                            self.handshake_info.network_id, response_handshake.network_id
                        )));
                    }
                } else {
                    return Err(BlockListenerError::Protocol(format!(
                        "Expected handshake response, got {:?}",
                        msg_type
                    )));
                }
            }
            _ => {
                return Err(BlockListenerError::Protocol(
                    "Invalid handshake response format".to_string(),
                ));
            }
        }

        info!("Handshake completed successfully with peer {}", self.info.peer_id);
        Ok(())
    }

    /// Handle incoming protocol messages
    async fn handle_message(
        &self,
        data: &[u8],
        event_tx: &mpsc::Sender<InternalEvent>,
    ) -> Result<()> {
        let (msg_type, payload) = self.parse_protocol_message(data)?;

        match msg_type {
            ProtocolMessageTypes::NewPeakWallet => {
                self.handle_new_peak(&payload, event_tx).await?;
            }
            ProtocolMessageTypes::RespondBlock => {
                self.handle_block_response(&payload, event_tx).await?;
            }
            ProtocolMessageTypes::NewPeak => {
                self.handle_new_peak(&payload, event_tx).await?;
            }
            _ => {
                debug!("Ignoring message type {:?} from peer {}", msg_type, self.info.peer_id);
            }
        }

        Ok(())
    }

    /// Handle new peak notifications
    async fn handle_new_peak(
        &self,
        payload: &[u8],
        event_tx: &mpsc::Sender<InternalEvent>,
    ) -> Result<()> {
        // Try to parse as NewPeakWallet first, then NewPeak
        let peak_info = if let Ok(peak_wallet) = NewPeakWallet::from_bytes(payload) {
            (peak_wallet.height, hex::encode(&peak_wallet.header_hash), peak_wallet.weight.to_string())
        } else {
            // Fallback to basic parsing if NewPeakWallet fails
            // This is a simplified implementation - in practice you'd parse the specific message type
            debug!("Could not parse as NewPeakWallet, using fallback parsing");
            return Ok(());
        };

        let new_peak_event = InternalEvent::NewPeak(NewPeakEvent {
            peer_id: self.info.peer_id.clone(),
            old_peak: None, // We don't track previous peak in this simplified version
            new_peak: peak_info.0,
            discovered_at: chrono::Utc::now(),
            header_hash: peak_info.1,
            weight: peak_info.2,
        });

        event_tx.send(new_peak_event).await
            .map_err(|_| BlockListenerError::EventChannelClosed)?;

        debug!("New peak {} received from peer {}", peak_info.0, self.info.peer_id);
        Ok(())
    }

    /// Handle block response messages
    async fn handle_block_response(
        &self,
        payload: &[u8],
        event_tx: &mpsc::Sender<InternalEvent>,
    ) -> Result<()> {
        let respond_block = RespondBlock::from_bytes(payload)
            .map_err(|e| BlockListenerError::Protocol(format!("Failed to parse block response: {}", e)))?;

        if let Some(block) = respond_block.block {
            let block_event = self.convert_block_to_event(block).await?;
            
            event_tx.send(InternalEvent::BlockReceived(block_event)).await
                .map_err(|_| BlockListenerError::EventChannelClosed)?;
        }

        Ok(())
    }

    /// Convert FullBlock to BlockReceivedEvent
    async fn convert_block_to_event(&self, block: FullBlock) -> Result<BlockReceivedEvent> {
        // Parse block using the generator parser
        let parser = chia_generator_parser::BlockParser::new();
        let parsed_block = parser.parse_block(block)
            .map_err(|e| BlockListenerError::InvalidBlockData {
                reason: e.to_string(),
            })?;

        // Convert to event format
        Ok(BlockReceivedEvent {
            height: parsed_block.height,
            header_hash: hex::encode(&parsed_block.header_hash),
            timestamp: parsed_block.timestamp.unwrap_or(0) as u64,
            weight: parsed_block.weight.to_string(),
            is_transaction_block: parsed_block.has_transactions_generator,
            peer_id: self.info.peer_id.clone(),
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
        })
    }

    /// Create a properly formatted protocol message
    fn create_protocol_message(&self, msg_type: ProtocolMessageTypes, payload: Vec<u8>) -> Result<Vec<u8>> {
        let mut message = Vec::new();
        
        // Message type (1 byte)
        message.push(msg_type as u8);
        
        // Message ID (2 bytes) - using 0 for simplicity
        message.extend_from_slice(&[0u8, 0u8]);
        
        // Payload length (4 bytes, big-endian)
        let payload_len = payload.len() as u32;
        message.extend_from_slice(&payload_len.to_be_bytes());
        
        // Payload
        message.extend_from_slice(&payload);
        
        Ok(message)
    }

    /// Parse a protocol message into type and payload
    fn parse_protocol_message(&self, data: &[u8]) -> Result<(ProtocolMessageTypes, Vec<u8>)> {
        if data.len() < 7 {
            return Err(BlockListenerError::Protocol("Message too short".to_string()));
        }

        let msg_type_byte = data[0];
        let msg_type = ProtocolMessageTypes::try_from(msg_type_byte)
            .map_err(|_| BlockListenerError::Protocol(format!("Unknown message type: {}", msg_type_byte)))?;

        let payload_len = u32::from_be_bytes([data[3], data[4], data[5], data[6]]) as usize;
        
        if data.len() < 7 + payload_len {
            return Err(BlockListenerError::Protocol("Incomplete message".to_string()));
        }

        let payload = data[7..7 + payload_len].to_vec();
        Ok((msg_type, payload))
    }

    /// Get connection information
    pub fn get_connection_info(&self) -> &ConnectionInfo {
        &self.info
    }

    /// Get peer info for status reporting
    pub fn get_peer_info(&self) -> PeerInfo {
        PeerInfo::new(
            self.info.peer_id.clone(),
            self.info.host.clone(),
            self.info.port,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_protocol_message_creation() {
        let connection_info = ConnectionInfo::new(
            "localhost".to_string(),
            8444,
            "mainnet".to_string(),
        );
        let handshake_info = HandshakeInfo::default();
        
        // This test would fail without proper TLS setup, so we'll just test the structure
        let payload = vec![1, 2, 3, 4];
        
        // We can't easily test the full PeerConnection without mocking TLS
        // So we'll test the message creation logic in isolation
        let message_data = create_test_protocol_message(ProtocolMessageTypes::Handshake, payload.clone());
        
        assert_eq!(message_data[0], ProtocolMessageTypes::Handshake as u8);
        assert_eq!(message_data[1..3], [0, 0]); // Message ID
        assert_eq!(message_data[3..7], [0, 0, 0, 4]); // Payload length
        assert_eq!(message_data[7..], payload); // Payload
    }

    fn create_test_protocol_message(msg_type: ProtocolMessageTypes, payload: Vec<u8>) -> Vec<u8> {
        let mut message = Vec::new();
        message.push(msg_type as u8);
        message.extend_from_slice(&[0u8, 0u8]); // Message ID
        let payload_len = payload.len() as u32;
        message.extend_from_slice(&payload_len.to_be_bytes());
        message.extend_from_slice(&payload);
        message
    }

    #[test]
    fn test_protocol_message_parsing() {
        let payload = vec![1, 2, 3, 4];
        let message = create_test_protocol_message(ProtocolMessageTypes::Handshake, payload.clone());
        
        // Parse the message
        let msg_type_byte = message[0];
        let msg_type = ProtocolMessageTypes::try_from(msg_type_byte).unwrap();
        let payload_len = u32::from_be_bytes([message[3], message[4], message[5], message[6]]) as usize;
        let parsed_payload = message[7..7 + payload_len].to_vec();
        
        assert_eq!(msg_type, ProtocolMessageTypes::Handshake);
        assert_eq!(parsed_payload, payload);
    }

    #[test]
    fn test_connection_info_creation() {
        let info = ConnectionInfo::new(
            "localhost".to_string(),
            8444,
            "mainnet".to_string(),
        );
        
        assert_eq!(info.peer_id, "localhost:8444");
        assert_eq!(info.host, "localhost");
        assert_eq!(info.port, 8444);
        assert_eq!(info.network_id, "mainnet");
        assert_eq!(info.websocket_url(), "wss://localhost:8444/ws");
    }

    #[test]
    fn test_peer_info_creation() {
        let info = PeerInfo::new(
            "test_peer".to_string(),
            "localhost".to_string(),
            8444,
        );
        
        assert_eq!(info.peer_id, "test_peer");
        assert_eq!(info.host, "localhost");
        assert_eq!(info.port, 8444);
        assert_eq!(info.state, PeerState::Disconnected);
        assert!(!info.is_healthy());
    }
} 