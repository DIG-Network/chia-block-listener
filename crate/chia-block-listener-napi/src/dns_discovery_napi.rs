use chia_block_listener::dns_discovery::{DiscoveryResult, DnsDiscoveryError, PeerAddress};
use chia_block_listener::DnsDiscoveryClient as CoreDnsDiscoveryClient;
use napi::bindgen_prelude::*;
use napi_derive::napi;
use tracing::{debug, info};

// Export error types for TypeScript
#[napi(object)]
#[derive(Clone)]
pub struct DnsDiscoveryErrorInfo {
    pub message: String,
    #[napi(js_name = "errorType")]
    pub error_type: String,
}

impl From<DnsDiscoveryError> for DnsDiscoveryErrorInfo {
    fn from(err: DnsDiscoveryError) -> Self {
        let error_type = match err {
            DnsDiscoveryError::ResolutionFailed(_) => "ResolutionFailed",
            DnsDiscoveryError::NoPeersFound => "NoPeersFound",
            DnsDiscoveryError::ResolverCreationFailed(_) => "ResolverCreationFailed",
        };

        Self {
            message: err.to_string(),
            error_type: error_type.to_string(),
        }
    }
}

// Export peer address for TypeScript
#[napi(object)]
#[derive(Clone)]
pub struct PeerAddressJS {
    pub host: String,
    pub port: u16,
    #[napi(js_name = "isIpv6")]
    pub is_ipv6: bool,
    #[napi(js_name = "displayAddress")]
    pub display_address: String,
}

impl From<&PeerAddress> for PeerAddressJS {
    fn from(peer: &PeerAddress) -> Self {
        Self {
            host: peer.host.to_string(),
            port: peer.port,
            is_ipv6: peer.is_ipv6,
            display_address: peer.display_address(),
        }
    }
}

// Export discovery result for TypeScript
#[napi(object)]
#[derive(Clone)]
pub struct DiscoveryResultJS {
    #[napi(js_name = "ipv4Peers")]
    pub ipv4_peers: Vec<PeerAddressJS>,
    #[napi(js_name = "ipv6Peers")]
    pub ipv6_peers: Vec<PeerAddressJS>,
    #[napi(js_name = "totalCount")]
    pub total_count: u32,
}

impl From<&DiscoveryResult> for DiscoveryResultJS {
    fn from(result: &DiscoveryResult) -> Self {
        Self {
            ipv4_peers: result.ipv4_peers.iter().map(|p| p.into()).collect(),
            ipv6_peers: result.ipv6_peers.iter().map(|p| p.into()).collect(),
            total_count: result.total_count as u32,
        }
    }
}

// Individual address result for resolve methods
#[napi(object)]
#[derive(Clone)]
pub struct AddressResult {
    pub addresses: Vec<String>,
    pub count: u32,
}

#[napi]
pub struct DnsDiscoveryClient {
    core: CoreDnsDiscoveryClient,
}

#[napi]
impl DnsDiscoveryClient {
    /// Create a new DNS discovery client
    #[napi(constructor)]
    pub fn new() -> Result<Self> {
        info!("Creating new DnsDiscoveryClient");

        let rt = tokio::runtime::Handle::current();
        let core = rt
            .block_on(async { CoreDnsDiscoveryClient::new().await })
            .map_err(|e| {
                let error_info = DnsDiscoveryErrorInfo::from(e);
                Error::new(Status::GenericFailure, error_info.message)
            })?;

        Ok(Self { core })
    }

    /// Discover peers for Chia mainnet
    #[napi(js_name = "discoverMainnetPeers")]
    pub async fn discover_mainnet_peers(&self) -> Result<DiscoveryResultJS> {
        debug!("Discovering mainnet peers via DNS");

        self.core
            .discover_mainnet_peers()
            .await
            .map(|result| DiscoveryResultJS::from(&result))
            .map_err(|e| {
                let error_info = DnsDiscoveryErrorInfo::from(e);
                Error::new(Status::GenericFailure, error_info.message)
            })
    }

    /// Discover peers for Chia testnet11
    #[napi(js_name = "discoverTestnet11Peers")]
    pub async fn discover_testnet11_peers(&self) -> Result<DiscoveryResultJS> {
        debug!("Discovering testnet11 peers via DNS");

        self.core
            .discover_testnet11_peers()
            .await
            .map(|result| DiscoveryResultJS::from(&result))
            .map_err(|e| {
                let error_info = DnsDiscoveryErrorInfo::from(e);
                Error::new(Status::GenericFailure, error_info.message)
            })
    }

    /// Discover peers using custom introducers
    #[napi(js_name = "discoverPeers")]
    pub async fn discover_peers(
        &self,
        introducers: Vec<String>,
        default_port: u16,
    ) -> Result<DiscoveryResultJS> {
        debug!(
            "Discovering peers using {} custom introducers",
            introducers.len()
        );

        self.core
            .discover_peers(&introducers, default_port)
            .await
            .map(|result| DiscoveryResultJS::from(&result))
            .map_err(|e| {
                let error_info = DnsDiscoveryErrorInfo::from(e);
                Error::new(Status::GenericFailure, error_info.message)
            })
    }

    /// Resolve IPv4 addresses (A records) for a hostname
    #[napi(js_name = "resolveIpv4")]
    pub async fn resolve_ipv4(&self, hostname: String) -> Result<AddressResult> {
        debug!("Resolving IPv4 addresses for {}", hostname);

        self.core
            .resolve_ipv4(hostname.as_str())
            .await
            .map(|addrs| AddressResult {
                addresses: addrs.iter().map(|addr| addr.to_string()).collect(),
                count: addrs.len() as u32,
            })
            .map_err(|e| {
                let error_info = DnsDiscoveryErrorInfo::from(e);
                Error::new(Status::GenericFailure, error_info.message)
            })
    }

    /// Resolve IPv6 addresses (AAAA records) for a hostname
    #[napi(js_name = "resolveIpv6")]
    pub async fn resolve_ipv6(&self, hostname: String) -> Result<AddressResult> {
        debug!("Resolving IPv6 addresses for {}", hostname);

        self.core
            .resolve_ipv6(hostname.as_str())
            .await
            .map(|addrs| AddressResult {
                addresses: addrs.iter().map(|addr| addr.to_string()).collect(),
                count: addrs.len() as u32,
            })
            .map_err(|e| {
                let error_info = DnsDiscoveryErrorInfo::from(e);
                Error::new(Status::GenericFailure, error_info.message)
            })
    }

    /// Resolve both IPv4 and IPv6 addresses for a hostname
    #[napi(js_name = "resolveBoth")]
    pub async fn resolve_both(&self, hostname: String, port: u16) -> Result<DiscoveryResultJS> {
        debug!("Resolving both IPv4 and IPv6 addresses for {}", hostname);

        self.core
            .resolve_both(hostname.as_str(), port)
            .await
            .map(|result| DiscoveryResultJS::from(&result))
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))
    }
}
