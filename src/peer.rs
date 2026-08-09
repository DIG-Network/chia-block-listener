use crate::{error::ChiaError, tls};
use chia_generator_parser::{parser::BlockParser, types::ParsedBlock};
use chia_protocol::{
    FullBlock, Handshake as ChiaHandshake, NewPeakWallet, NodeType, ProtocolMessageTypes,
    RequestBlock, RespondBlock,
};
use chia_traits::Streamable;
use futures_util::{SinkExt, StreamExt};
use std::net::IpAddr;
use tokio::net::TcpStream;
use tokio::sync::mpsc;
use tokio_tungstenite::{
    connect_async_tls_with_config, tungstenite::protocol::CloseFrame,
    tungstenite::Message as WsMessage, Connector, MaybeTlsStream, WebSocketStream,
};
use tokio_util::sync::CancellationToken;
use tracing::{debug, error, info, trace, warn};

type WebSocket = WebSocketStream<MaybeTlsStream<TcpStream>>;

/// Renders a peer's WebSocket close frame for logs and error strings.
///
/// The reason text is authored by the remote peer — tungstenite only checks it
/// is valid UTF-8 — so it is rendered with `{:?}`, which escapes control
/// characters. Rendered with `{}`, a single `\n` in the reason would let a peer
/// forge extra lines in our logs and in the JS `Error.message` this string
/// eventually reaches.
fn describe_close_reason(frame: Option<&CloseFrame<'_>>) -> String {
    frame.map_or_else(
        || "no close frame".to_string(),
        |f| format!("code {} - {:?}", f.code, f.reason),
    )
}

/// The longest peer-authored string rendered into a log line or an error.
///
/// A handshake is an ordinary data message, so its strings are bounded only by
/// the 64 MiB WebSocket frame limit — three orders of magnitude above anything
/// a real `network_id` or version needs, and the resulting error is logged once
/// per reconnect attempt.
const MAX_PEER_STRING: usize = 64;

/// The longest rendering `describe_peer_string` can return, in bytes.
///
/// `MAX_PEER_STRING` bounds *characters*, and `{:?}` expands a non-printable
/// character to a `\u{...}` escape of up to ten bytes, so the byte length of a
/// rendering is several times its character count. Stated here because the
/// character bound alone reads as a byte bound and is not one:
/// `the_peer_string_bound_is_in_characters_not_bytes` pins the difference.
///
/// Derived rather than enforced: the bound follows from the character limit,
/// so it exists to be asserted against, not to be applied.
#[cfg(test)]
const MAX_PEER_STRING_RENDERED_BYTES: usize = MAX_PEER_STRING * 10 + r#""" (truncated)"#.len();

/// Renders a peer-authored string for a log line or an error message.
///
/// Two hazards, both closed here. `{:?}` escapes control characters, so a `\n`
/// cannot forge a log line — the same treatment `describe_close_reason` gives a
/// close frame. Truncation bounds the volume, which a close frame did not need:
/// tungstenite caps a close reason at 123 bytes, while a handshake field is
/// capped only by the frame size.
///
/// The truncation bound is in **characters**, not bytes — see
/// `MAX_PEER_STRING_RENDERED_BYTES` for what that costs in the worst case.
fn describe_peer_string(value: &str) -> String {
    let truncated: String = value.chars().take(MAX_PEER_STRING).collect();
    if truncated.len() < value.len() {
        format!("{truncated:?} (truncated)")
    } else {
        format!("{truncated:?}")
    }
}

/// The substring `peer_pool` matches to evict a peer that answered the wrong
/// block. Kept as a constant so the guard and the eviction rule cannot drift.
pub(crate) const BLOCK_HEIGHT_MISMATCH: &str = "Block height mismatch";

/// Accepts a `RespondBlock` only when it answers the request it claims to, and
/// yields the height actually delivered.
///
/// A `RequestBlock` carries a height and nothing else identifies the response,
/// so without this check any parseable block satisfies any request. That is not
/// merely untidy: `peer_pool` records a fetched height as a *locally observed*
/// peak, trusted with no corroboration because "we parsed it ourselves". A peer
/// free to answer height 1 to a request for height 10_000_000 would therefore
/// set the trusted peak to a height that does not exist. A peer answering a
/// height other than the one asked for is misbehaving whatever it is aimed at,
/// so this is a first-class protocol error.
///
/// **What this buys, and what it does not.** It is not verification: nothing
/// here checks the block against a proof of space or time, a signature, or a
/// second source. What it buys is a *bound* — `local_peak` cannot exceed the
/// highest height this process asked for. Within this crate that holds,
/// because a requested height originates only from the consumer and never from
/// a peer announcement. A consumer that fetches the height it just learned
/// from `NewPeakHeight` or `get_highest_peak` hands a peer-influenced height
/// back to peers and collapses the bound to `pool_peak`'s two-endpoint bar.
/// The guarantee is only as strong as the caller's choice of height.
fn accepted_block_height(requested: u64, block: &FullBlock) -> Result<u32, ChiaError> {
    let delivered = block.reward_chain_block.height;
    if u64::from(delivered) != requested {
        return Err(ChiaError::Protocol(format!(
            "{BLOCK_HEIGHT_MISMATCH}: requested height {requested}, peer answered with height {delivered}"
        )));
    }
    Ok(delivered)
}

/// Builds the error reported when a peer closes the socket mid-request.
///
/// The word "closed" is load-bearing: `peer_pool` substring-matches it on the
/// `ChiaError::Connection` text to decide whether to evict the peer.
fn peer_closed_during_request(reason: &str) -> ChiaError {
    ChiaError::Connection(format!(
        "Peer closed connection during block request ({reason})"
    ))
}

#[derive(Clone)]
pub struct PeerConnection {
    host: String,
    port: u16,
    network_id: String,
}

#[derive(Clone, Debug)]
pub enum StreamEvent {
    ParsedBlock(ParsedBlock),
    NewPeak(u32),
}

impl PeerConnection {
    pub fn new(host: String, port: u16, network_id: String) -> Self {
        Self {
            host,
            port,
            network_id,
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    pub async fn connect(&self) -> Result<WebSocket, ChiaError> {
        info!("Connecting to peer at {}:{}", self.host, self.port);

        // Load or generate certificates
        let cert = tls::load_or_generate_cert()?;
        let tls_connector = tls::create_tls_connector(&cert)?;
        let connector = Connector::NativeTls(tls_connector);

        // Check if the host is an IPv6 address and format accordingly
        // IPv6 addresses need to be wrapped in brackets when used in URLs
        let host_formatted = match self.host.parse::<IpAddr>() {
            Ok(IpAddr::V6(_)) => {
                // This is an IPv6 address, wrap it in brackets
                format!("[{}]", self.host)
            }
            _ => {
                // IPv4 address or hostname, use as-is
                self.host.clone()
            }
        };

        let url = format!("wss://{}:{}/ws", host_formatted, self.port);
        debug!("WebSocket URL: {}", url);

        let (ws_stream, _) = connect_async_tls_with_config(&url, None, false, Some(connector))
            .await
            .map_err(|e| ChiaError::WebSocket(Box::new(e)))?;

        info!("WebSocket connection established to {}", self.host);
        Ok(ws_stream)
    }

    pub async fn handshake(&self, ws_stream: &mut WebSocket) -> Result<(), ChiaError> {
        info!("Performing Chia handshake with {}", self.host);

        // Send our handshake - matching SDK exactly
        let handshake = ChiaHandshake {
            network_id: self.network_id.clone(),
            protocol_version: "0.0.37".to_string(),
            software_version: "0.0.0".to_string(),
            server_port: 0,              // 0 for wallet clients
            node_type: NodeType::Wallet, // Connect as wallet
            capabilities: vec![
                (1, "1".to_string()), // BASE
                (2, "1".to_string()), // BLOCK_HEADERS
                (3, "1".to_string()), // RATE_LIMITS_V2
            ],
        };

        // Serialize and send handshake
        let handshake_bytes = handshake
            .to_bytes()
            .map_err(|e| ChiaError::Protocol(e.to_string()))?;

        let message = chia_protocol::Message {
            msg_type: ProtocolMessageTypes::Handshake,
            id: None,
            data: handshake_bytes.into(),
        };

        let message_bytes = message
            .to_bytes()
            .map_err(|e| ChiaError::Protocol(e.to_string()))?;

        ws_stream
            .send(WsMessage::Binary(message_bytes))
            .await
            .map_err(|e| ChiaError::WebSocket(Box::new(e)))?;

        // Wait for peer's handshake
        if let Some(msg) = ws_stream.next().await {
            match msg {
                Ok(WsMessage::Binary(data)) => {
                    let response = chia_protocol::Message::from_bytes(&data)
                        .map_err(|e| ChiaError::Protocol(e.to_string()))?;

                    if response.msg_type == ProtocolMessageTypes::Handshake {
                        // Parse and validate peer's handshake
                        let peer_handshake = ChiaHandshake::from_bytes(&response.data)
                            .map_err(|e| ChiaError::Protocol(e.to_string()))?;

                        if peer_handshake.node_type != NodeType::FullNode {
                            return Err(ChiaError::Protocol(format!(
                                "Expected FullNode, got {:?}",
                                peer_handshake.node_type
                            )));
                        }

                        if peer_handshake.network_id != self.network_id {
                            return Err(ChiaError::Protocol(format!(
                                "Network ID mismatch: expected {}, got {}",
                                self.network_id,
                                describe_peer_string(&peer_handshake.network_id)
                            )));
                        }

                        info!(
                            "Handshake successful with {} (protocol: {})",
                            self.host,
                            describe_peer_string(&peer_handshake.protocol_version)
                        );
                        Ok(())
                    } else {
                        Err(ChiaError::Protocol(format!(
                            "Expected handshake, got message type {:?}",
                            response.msg_type
                        )))
                    }
                }
                Ok(WsMessage::Close(_)) => Err(ChiaError::Connection(
                    "Peer closed connection during handshake".to_string(),
                )),
                Ok(_) => Err(ChiaError::Protocol("Unexpected message type".to_string())),
                Err(e) => Err(ChiaError::WebSocket(Box::new(e))),
            }
        } else {
            Err(ChiaError::Connection(
                "Connection closed during handshake".to_string(),
            ))
        }
    }

    pub async fn listen_for_blocks(
        mut ws_stream: WebSocket,
        event_sender: mpsc::Sender<StreamEvent>,
        cancel: CancellationToken,
    ) -> Result<(), ChiaError> {
        info!("Listening for blocks and messages");

        loop {
            let next_msg = tokio::select! {
                _ = cancel.cancelled() => break,
                msg = ws_stream.next() => msg,
            };

            let Some(msg) = next_msg else {
                break;
            };
            match msg {
                Ok(WsMessage::Binary(data)) => {
                    match chia_protocol::Message::from_bytes(&data) {
                        Ok(message) => {
                            trace!("Received message type: {:?}", message.msg_type);

                            match message.msg_type {
                                ProtocolMessageTypes::NewPeakWallet => {
                                    if let Ok(new_peak) = NewPeakWallet::from_bytes(&message.data) {
                                        debug!(
                                            "New peak at height {} from wallet perspective",
                                            new_peak.height
                                        );

                                        // Emit new peak notification (best-effort)
                                        if let Err(e) = event_sender
                                            .try_send(StreamEvent::NewPeak(new_peak.height))
                                        {
                                            warn!("Failed to queue new peak event: {}", e);
                                        }

                                        // Request the full block
                                        let request = RequestBlock {
                                            height: new_peak.height,
                                            include_transaction_block: true,
                                        };

                                        if let Ok(request_bytes) = request.to_bytes() {
                                            let request_msg = chia_protocol::Message {
                                                msg_type: ProtocolMessageTypes::RequestBlock,
                                                id: Some(1), // Add request ID
                                                data: request_bytes.into(),
                                            };

                                            if let Ok(msg_bytes) = request_msg.to_bytes() {
                                                if let Err(e) = ws_stream
                                                    .send(WsMessage::Binary(msg_bytes))
                                                    .await
                                                {
                                                    error!("Failed to request block: {}", e);
                                                }
                                            }
                                        }
                                    }
                                }

                                ProtocolMessageTypes::NewPeak => {
                                    // This is for full nodes - we might see this too
                                    trace!("Received NewPeak (full node message)");
                                }

                                ProtocolMessageTypes::RespondBlock => {
                                    match RespondBlock::from_bytes(&message.data) {
                                        Ok(respond_block) => {
                                            let block = respond_block.block;
                                            debug!(
                                                "Received block at height {}",
                                                block.reward_chain_block.height
                                            );

                                            // Parse the block using chia-generator-parser
                                            match Self::parse_block(block).await {
                                                Ok(parsed_block) => {
                                                    if let Err(e) = event_sender
                                                        .send(StreamEvent::ParsedBlock(
                                                            parsed_block,
                                                        ))
                                                        .await
                                                    {
                                                        error!(
                                                            "Failed to send parsed block through channel: {}",
                                                            e
                                                        );
                                                        break;
                                                    }
                                                }
                                                Err(e) => {
                                                    error!("Failed to parse block: {}", e);
                                                }
                                            }
                                        }
                                        Err(e) => {
                                            error!("Failed to parse RespondBlock: {}", e);
                                        }
                                    }
                                }

                                ProtocolMessageTypes::CoinStateUpdate => {
                                    trace!("Received coin state update");
                                }

                                _ => {
                                    trace!("Received other message type: {:?}", message.msg_type);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to parse message: {}", e);
                        }
                    }
                }
                Ok(WsMessage::Close(frame)) => {
                    info!("Peer closed connection: {:?}", frame);
                    break;
                }
                Ok(WsMessage::Ping(data)) => {
                    // Respond to ping
                    if let Err(e) = ws_stream.send(WsMessage::Pong(data)).await {
                        error!("Failed to send pong: {}", e);
                    }
                }
                Ok(_) => {
                    // Ignore other message types
                }
                Err(e) => {
                    error!("WebSocket error: {}", e);
                    return Err(ChiaError::WebSocket(Box::new(e)));
                }
            }
        }

        info!("Connection closed");
        Ok(())
    }

    /// Parse a FullBlock using chia-generator-parser
    async fn parse_block(block: FullBlock) -> Result<ParsedBlock, ChiaError> {
        debug!(
            "Parsing block at height {}",
            block.reward_chain_block.height
        );

        // Use chia-generator-parser to parse the block directly
        let parser = BlockParser::new();
        let parsed_block = parser
            .parse_full_block(&block)
            .map_err(|e| ChiaError::Protocol(e.to_string()))?;

        debug!(
            "Parsed block {}: {} coin additions, {} coin removals, {} coin spends, generator: {}",
            parsed_block.height,
            parsed_block.coin_additions.len(),
            parsed_block.coin_removals.len(),
            parsed_block.coin_spends.len(),
            parsed_block.has_transactions_generator
        );

        Ok(parsed_block)
    }

    pub async fn request_block_by_height(
        &self,
        height: u64,
        ws_stream: &mut WebSocket,
    ) -> Result<FullBlock, ChiaError> {
        debug!("Requesting block at height {}", height);

        // `RequestBlock` carries a `u32`, so a larger height has no wire
        // representation. Truncating it would ask for `height % 2^32` and then
        // reject every honest answer as a height mismatch, evicting peers for
        // answering correctly.
        let requested = u32::try_from(height).map_err(|_| {
            ChiaError::Protocol(format!(
                "Requested block height {height} exceeds the protocol maximum of {}",
                u32::MAX
            ))
        })?;

        let request = RequestBlock {
            height: requested,
            include_transaction_block: true,
        };

        let request_bytes = request
            .to_bytes()
            .map_err(|e| ChiaError::Protocol(e.to_string()))?;

        let request_msg = chia_protocol::Message {
            msg_type: ProtocolMessageTypes::RequestBlock,
            id: Some(1), // Add request ID
            data: request_bytes.into(),
        };

        let request_bytes = request_msg
            .to_bytes()
            .map_err(|e| ChiaError::Protocol(e.to_string()))?;

        ws_stream
            .send(WsMessage::Binary(request_bytes))
            .await
            .map_err(|e| ChiaError::WebSocket(Box::new(e)))?;

        // Wait for the response, handling other messages in between
        let mut attempts = 0;
        const MAX_ATTEMPTS: u32 = 100; // Prevent infinite loops

        while attempts < MAX_ATTEMPTS {
            attempts += 1;

            if let Some(msg) = ws_stream.next().await {
                match msg {
                    Ok(WsMessage::Binary(data)) => {
                        match chia_protocol::Message::from_bytes(&data) {
                            Ok(response) => {
                                trace!(
                                    "Received message type: {:?} while waiting for block",
                                    response.msg_type
                                );

                                match response.msg_type {
                                    ProtocolMessageTypes::RespondBlock => {
                                        match RespondBlock::from_bytes(&response.data) {
                                            Ok(respond_block) => {
                                                let block = respond_block.block;
                                                debug!(
                                                    "Received block at height {} - transactions_generator: {} bytes, has_foliage_transaction_block: {}",
                                                    block.reward_chain_block.height,
                                                    block.transactions_generator.as_ref().map(|g| g.len()).unwrap_or(0),
                                                    block.foliage_transaction_block.is_some()
                                                );
                                                accepted_block_height(height, &block)?;
                                                return Ok(block);
                                            }
                                            Err(e) => {
                                                error!("Failed to parse RespondBlock: {}", e);
                                                return Err(ChiaError::Protocol(e.to_string()));
                                            }
                                        }
                                    }
                                    ProtocolMessageTypes::RejectBlock => {
                                        error!("Block request rejected by peer");
                                        return Err(ChiaError::Protocol(
                                            "Block request rejected".to_string(),
                                        ));
                                    }
                                    ProtocolMessageTypes::NewPeakWallet => {
                                        // Just log and continue waiting for our response
                                        if let Ok(new_peak) =
                                            NewPeakWallet::from_bytes(&response.data)
                                        {
                                            trace!("Received NewPeakWallet at height {} while waiting for block", new_peak.height);
                                        }
                                        continue;
                                    }
                                    ProtocolMessageTypes::CoinStateUpdate => {
                                        trace!("Received CoinStateUpdate while waiting for block");
                                        continue;
                                    }
                                    _ => {
                                        trace!("Received other message type while waiting for block: {:?}", response.msg_type);
                                        continue;
                                    }
                                }
                            }
                            Err(e) => {
                                warn!("Failed to parse message while waiting for block: {}", e);
                                continue;
                            }
                        }
                    }
                    Ok(WsMessage::Close(frame)) => {
                        // The close frame carries the peer's reason. Dropping it
                        // turns every refusal into one indistinguishable string,
                        // which is what made this failure mode hard to diagnose.
                        let reason = describe_close_reason(frame.as_ref());
                        error!("Peer closed connection during block request: {}", reason);
                        return Err(peer_closed_during_request(&reason));
                    }
                    Ok(WsMessage::Ping(data)) => {
                        // Respond to ping
                        if let Err(e) = ws_stream.send(WsMessage::Pong(data)).await {
                            error!("Failed to send pong: {}", e);
                        }
                        continue;
                    }
                    Ok(_) => {
                        trace!("Unexpected WebSocket message type during block request");
                        continue;
                    }
                    Err(e) => {
                        error!("WebSocket error: {}", e);
                        return Err(ChiaError::WebSocket(Box::new(e)));
                    }
                }
            } else {
                error!("Connection closed during block request");
                return Err(ChiaError::Connection(
                    "Connection closed during block request".to_string(),
                ));
            }
        }

        error!(
            "Timeout waiting for block response after {} attempts",
            MAX_ATTEMPTS
        );
        Err(ChiaError::Protocol(
            "Timeout waiting for block response".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use arbitrary::Arbitrary;
    use tokio_tungstenite::tungstenite::protocol::frame::coding::CloseCode;

    fn close_frame(reason: &str) -> CloseFrame<'_> {
        CloseFrame {
            code: CloseCode::Policy,
            reason: reason.into(),
        }
    }

    /// A peer authors the close reason, so control characters in it must never
    /// reach a log line or an error string verbatim.
    #[test]
    fn close_reason_control_characters_are_escaped() {
        let hostile = "bye\r\nERROR forged log line\u{1b}[31m\u{0}";

        let rendered = describe_close_reason(Some(&close_frame(hostile)));

        for raw in ['\n', '\r', '\u{1b}', '\u{0}'] {
            assert!(
                !rendered.contains(raw),
                "raw {raw:?} survived rendering: {rendered:?}"
            );
        }
        assert!(rendered.contains("forged log line"), "{rendered}");
    }

    /// `peer_pool` substring-matches "closed" on the resulting error string to
    /// decide whether to evict a peer; escaping must not disturb that, even when
    /// the peer deliberately crafts a reason to break the match.
    #[test]
    fn peer_closed_error_stays_matchable_and_escaped() {
        let reason = describe_close_reason(Some(&close_frame("closed\r\nnot really")));

        let message = peer_closed_during_request(&reason).to_string();

        assert!(message.contains("closed"), "{message}");
        assert!(!message.contains('\n'), "{message}");
        assert!(!message.contains('\r'), "{message}");
    }

    #[test]
    fn missing_close_frame_is_described() {
        assert_eq!(describe_close_reason(None), "no close frame");
    }

    /// Builds a real `FullBlock` sitting at `height`.
    ///
    /// Every other field is filled from an all-zero `Arbitrary` source: the
    /// guard under test reads exactly one field, and a peer's block is
    /// otherwise attacker-shaped anyway.
    fn block_at_height(height: u32) -> FullBlock {
        let zeros = [0u8; 4096];
        let mut source = arbitrary::Unstructured::new(&zeros);
        let mut block = FullBlock::arbitrary(&mut source).expect("arbitrary FullBlock");
        block.reward_chain_block.height = height;
        block
    }

    /// The block a peer sends back is only identified by the height it carries,
    /// so a peer that answers a *different* height than asked has answered a
    /// question nobody put to it. Height 1 is a real, long-final block: the
    /// cheapest thing a hostile peer can hand back to a request for a height
    /// that does not exist yet.
    #[test]
    fn a_block_from_a_different_height_is_rejected() {
        let error = accepted_block_height(10_000_000, &block_at_height(1))
            .expect_err("a block from height 1 must not answer a request for height 10_000_000");

        let message = error.to_string();
        assert!(message.contains(BLOCK_HEIGHT_MISMATCH), "{message}");
        assert!(message.contains("10000000"), "{message}");
        assert!(message.contains('1'), "{message}");
    }

    /// The mismatch is rejected in both directions: a peer running ahead of the
    /// request is as unidentified as one running behind it.
    #[test]
    fn a_block_from_a_later_height_is_also_rejected() {
        assert!(accepted_block_height(100, &block_at_height(101)).is_err());
    }

    /// The height the caller may record is the one the block carries, not the
    /// one the caller asked for. Asserting the returned value — rather than
    /// only `is_ok()` — is what keeps the provenance in the block.
    #[test]
    fn a_matching_block_yields_the_height_it_carries() {
        assert_eq!(
            accepted_block_height(7_654_321, &block_at_height(7_654_321)).unwrap(),
            7_654_321
        );
    }

    /// A handshake field is peer-authored and, unlike a close reason, bounded
    /// only by the WebSocket frame size. The rendering must neither let a peer
    /// forge log lines nor let it choose how much of the log it occupies.
    #[test]
    fn peer_strings_are_escaped_and_bounded() {
        let hostile = format!("mainnet\r\nERROR forged{}", "A".repeat(10_000));

        let rendered = describe_peer_string(&hostile);

        assert!(!rendered.contains('\n'), "{rendered}");
        assert!(!rendered.contains('\r'), "{rendered}");
        assert!(
            rendered.len() < 128,
            "a peer chose {} bytes of log line",
            rendered.len()
        );
        assert!(rendered.contains("truncated"), "{rendered}");
    }

    /// An ordinary value must survive intact, or the bound is being paid for
    /// with unreadable logs.
    #[test]
    fn an_ordinary_peer_string_is_rendered_whole() {
        assert_eq!(describe_peer_string("mainnet"), "\"mainnet\"");
    }

    /// The bound is pinned from both sides: at the limit nothing is dropped,
    /// one character over is truncated.
    #[test]
    fn the_peer_string_bound_holds_at_the_limit() {
        let at_limit = "a".repeat(MAX_PEER_STRING);
        assert!(!describe_peer_string(&at_limit).contains("truncated"));

        let one_over = "a".repeat(MAX_PEER_STRING + 1);
        assert!(describe_peer_string(&one_over).contains("truncated"));
    }

    // ---- Seam coverage -------------------------------------------------
    //
    // The guards above are exercised as functions. These tests exercise them
    // where production calls them: over a real WebSocket, against a peer whose
    // answers the test writes. Deleting a *call site* -- as opposed to
    // neutering the function -- is a mutation the direct tests cannot see.

    use tokio::net::TcpListener;
    use tokio_tungstenite::{accept_async, client_async};

    /// A real WebSocket pair over loopback TCP: the client half is exactly the
    /// `WebSocket` production speaks, the server half is the hostile peer.
    async fn connected_pair() -> (WebSocket, WebSocketStream<TcpStream>) {
        let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind");
        let addr = listener.local_addr().expect("addr");

        let server = tokio::spawn(async move {
            let (tcp, _) = listener.accept().await.expect("accept");
            accept_async(tcp).await.expect("server handshake")
        });

        let tcp = TcpStream::connect(addr).await.expect("connect");
        let (client, _response) =
            client_async(format!("ws://{addr}/ws"), MaybeTlsStream::Plain(tcp))
                .await
                .expect("client handshake");

        (client, server.await.expect("server task"))
    }

    /// Reads the peer's `RequestBlock`, answers it with `block`, and yields the
    /// height that was actually asked for.
    async fn answer_block_request(
        server: &mut WebSocketStream<TcpStream>,
        block: FullBlock,
    ) -> u32 {
        let raw = server.next().await.expect("a request").expect("a frame");
        let WsMessage::Binary(bytes) = raw else {
            panic!("expected a binary request, got {raw:?}");
        };
        let requested = RequestBlock::from_bytes(
            &chia_protocol::Message::from_bytes(&bytes)
                .expect("request message")
                .data,
        )
        .expect("RequestBlock")
        .height;

        let response = chia_protocol::Message {
            msg_type: ProtocolMessageTypes::RespondBlock,
            id: Some(1),
            data: RespondBlock { block }
                .to_bytes()
                .expect("RespondBlock")
                .into(),
        };
        server
            .send(WsMessage::Binary(response.to_bytes().expect("message")))
            .await
            .expect("send response");

        requested
    }

    fn peer() -> PeerConnection {
        PeerConnection::new("127.0.0.1".to_string(), 8444, "mainnet".to_string())
    }

    /// The guard has to be *wired in*. A peer answering a request for a height
    /// far ahead of the chain with long-final block 1 must fail the request as
    /// a protocol error, through the real request path.
    #[tokio::test]
    async fn a_wrong_height_answer_is_rejected_through_the_socket() {
        let (mut client, mut server) = connected_pair().await;
        let hostile = tokio::spawn(async move {
            answer_block_request(&mut server, block_at_height(1)).await;
            server
        });

        let error = peer()
            .request_block_by_height(10_000_000, &mut client)
            .await
            .expect_err("block 1 must not answer a request for height 10_000_000");

        assert!(error.to_string().contains(BLOCK_HEIGHT_MISMATCH), "{error}");
        hostile.await.expect("hostile peer task");
    }

    /// The control the rejection above needs: an honest answer still succeeds,
    /// so the seam test cannot pass by failing every request.
    #[tokio::test]
    async fn a_matching_answer_is_accepted_through_the_socket() {
        let (mut client, mut server) = connected_pair().await;
        let hostile = tokio::spawn(async move {
            answer_block_request(&mut server, block_at_height(7_654_321)).await
        });

        let block = peer()
            .request_block_by_height(7_654_321, &mut client)
            .await
            .expect("an honest answer must satisfy the request");

        assert_eq!(block.reward_chain_block.height, 7_654_321);
        assert_eq!(hostile.await.expect("hostile peer task"), 7_654_321);
    }

    /// `RequestBlock` carries a `u32`, so a height that does not fit has no
    /// representation on the wire. Truncating it instead of rejecting it would
    /// ask for `height % 2^32` and then reject every honest answer as a
    /// mismatch -- evicting honest peers for answering correctly. Unreachable
    /// through the napi surface, which takes a `u32`; reachable from the Rust
    /// API.
    #[tokio::test]
    async fn a_height_above_the_protocol_maximum_is_rejected_before_the_wire() {
        let (mut client, mut server) = connected_pair().await;
        let too_high = u64::from(u32::MAX) + 1;

        // Bounded: without the guard the truncated height goes on the wire and
        // this call blocks forever waiting for an answer nobody will send, so
        // the timeout is what turns that regression into a failure rather than
        // a hung suite.
        let error = tokio::time::timeout(
            std::time::Duration::from_secs(5),
            peer().request_block_by_height(too_high, &mut client),
        )
        .await
        .expect("the height must be rejected without waiting on the peer")
        .expect_err("a height with no wire representation must not be requested");

        assert!(error.to_string().contains(&too_high.to_string()), "{error}");
        assert!(
            tokio::time::timeout(std::time::Duration::from_millis(200), server.next())
                .await
                .is_err(),
            "the request reached the peer instead of being rejected up front"
        );
    }

    /// The bound pinned from the other side: the largest representable height
    /// is requested verbatim, not rejected and not truncated.
    #[tokio::test]
    async fn the_largest_representable_height_is_requested_verbatim() {
        let (mut client, mut server) = connected_pair().await;
        let hostile = tokio::spawn(async move {
            answer_block_request(&mut server, block_at_height(u32::MAX)).await
        });

        let block = peer()
            .request_block_by_height(u64::from(u32::MAX), &mut client)
            .await
            .expect("the largest representable height must be requestable");

        assert_eq!(block.reward_chain_block.height, u32::MAX);
        assert_eq!(hostile.await.expect("hostile peer task"), u32::MAX);
    }

    /// The handshake's rendering is wired in too: a hostile `network_id`
    /// reaches the error message only through `describe_peer_string`, so the
    /// escaping and the bound hold on the real path.
    #[tokio::test]
    async fn a_hostile_network_id_is_escaped_through_the_handshake() {
        let (mut client, mut server) = connected_pair().await;
        let hostile_id = format!("wrongnet\r\nERROR forged{}", "A".repeat(10_000));

        let hostile = tokio::spawn(async move {
            let _ours = server.next().await.expect("our handshake").expect("frame");
            let handshake = ChiaHandshake {
                network_id: hostile_id,
                protocol_version: "0.0.37".to_string(),
                software_version: "1.0.0".to_string(),
                server_port: 8444,
                node_type: NodeType::FullNode,
                capabilities: vec![],
            };
            let message = chia_protocol::Message {
                msg_type: ProtocolMessageTypes::Handshake,
                id: None,
                data: handshake.to_bytes().expect("handshake").into(),
            };
            server
                .send(WsMessage::Binary(message.to_bytes().expect("message")))
                .await
                .expect("send handshake");
            server
        });

        let error = peer()
            .handshake(&mut client)
            .await
            .expect_err("a mismatched network id must fail the handshake");

        let message = error.to_string();
        assert!(!message.contains('\n'), "{message}");
        assert!(!message.contains('\r'), "{message}");
        assert!(
            message.len() < 256,
            "a peer chose {} bytes of error message",
            message.len()
        );
        hostile.await.expect("hostile peer task");
    }

    /// The bound `describe_peer_string` enforces is in *characters*, and `{:?}`
    /// expands a non-printable character to a `\u{...}` escape -- so the byte
    /// length of the rendering is several times the character count.
    /// `peer_strings_are_escaped_and_bounded` asserts `< 128` bytes only
    /// because its input is ASCII; this pins what the bound really costs, so
    /// neither the doc nor that test can be read as promising a byte bound.
    #[test]
    fn the_peer_string_bound_is_in_characters_not_bytes() {
        let rendered = describe_peer_string(&"\u{10fffe}".repeat(1_000));

        assert!(
            rendered.len() > 128,
            "escaping was expected to exceed the ASCII-case byte length: {}",
            rendered.len()
        );
        assert!(
            rendered.len() <= MAX_PEER_STRING_RENDERED_BYTES,
            "a peer chose {} bytes of log line",
            rendered.len()
        );
    }
}
