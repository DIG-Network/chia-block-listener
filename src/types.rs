// Rust-native event and data types used by the core crate
#[derive(Clone, Debug)]
pub struct PeerConnectedEvent {
    pub peer_id: String,
    pub host: String,
    pub port: u32,
}

#[derive(Clone, Debug)]
pub struct PeerDisconnectedEvent {
    pub peer_id: String,
    pub host: String,
    pub port: u32,
    pub message: Option<String>,
}

/// Emitted when the pool's peak height *rises*.
///
/// The peak is a property of the pool, not of any one peer: it is the highest
/// height that either two pool entries independently claim, or that this
/// process requested a block for and received a matching block at. So this
/// event says the pool's view of the chain moved forward — never that the peer
/// named in it announced anything in particular.
///
/// A falling peak — a peer evicted, a streaming connection closed — is not
/// emitted; it is visible only by polling `get_highest_peak`. Emission is
/// on-rise-only so that a peer cannot drive the event channel.
#[derive(Clone, Debug)]
pub struct NewPeakHeightEvent {
    /// The pool peak last announced by this event, or `None` if this is the
    /// first announcement or the pool peak had since become uncorroborated.
    /// It is the previously *announced* value, not a previous claim by
    /// `peer_id`, and it may be lower than the peak reported between the two
    /// events.
    pub old_peak: Option<u32>,
    /// The pool peak now. Strictly greater than `old_peak` when that is set.
    pub new_peak: u32,
    /// The peer whose observation triggered the recomputation. This peer is
    /// the *occasion* for the event, not its source: it need never have
    /// claimed `new_peak`, which is generally corroborated by other entries.
    pub peer_id: String,
}

#[derive(Clone, Debug)]
pub struct CoinRecord {
    pub parent_coin_info: String,
    pub puzzle_hash: String,
    pub amount: String,
}

#[derive(Clone, Debug)]
pub struct CoinSpend {
    pub coin: CoinRecord,
    pub puzzle_reveal: String,
    pub solution: String,
    pub offset: u32,
}

#[derive(Clone, Debug)]
pub struct BlockReceivedEvent {
    pub peer_id: String,
    pub height: u32,
    pub weight: String,
    pub header_hash: String,
    pub timestamp: u32,
    pub coin_additions: Vec<CoinRecord>,
    pub coin_removals: Vec<CoinRecord>,
    pub coin_spends: Vec<CoinSpend>,
    pub coin_creations: Vec<CoinRecord>,
    pub has_transactions_generator: bool,
    pub generator_size: u32,
}

// Unified event enum for Rust consumers (Listener facade)
#[derive(Clone, Debug)]
pub enum Event {
    PeerConnected(PeerConnectedEvent),
    PeerDisconnected(PeerDisconnectedEvent),
    NewPeakHeight(NewPeakHeightEvent),
    BlockReceived(BlockReceivedEvent),
}

// Configuration for Listener / event buffering
#[derive(Clone, Debug)]
pub struct BlockListenerConfig {
    pub buffer: usize, // ring buffer size per subscriber
    pub auto_reconnect: bool,
    pub network_id: String,
    pub default_port: u16,
    pub max_auto_reconnect_retries: u32,
}

impl Default for BlockListenerConfig {
    fn default() -> Self {
        Self {
            buffer: 1024,
            auto_reconnect: false,
            network_id: "mainnet".to_string(),
            default_port: 8444,
            max_auto_reconnect_retries: 10,
        }
    }
}
