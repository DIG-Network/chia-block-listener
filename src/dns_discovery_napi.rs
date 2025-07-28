use dns_discovery::{DiscoveryResult, DnsDiscovery, DnsDiscoveryError, PeerAddress};
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

// Export peer address for TypeScript with clean name
#[napi(object, js_name = "PeerAddress")]
#[derive(Clone)]
pub struct PeerAddressWrapper {
    pub host: String,
    pub port: u16,
    #[napi(js_name = "isIpv6")]
    pub is_ipv6: bool,
    #[napi(js_name = "displayAddress")]
    pub display_address: String,
}

impl From<&PeerAddress> for PeerAddressWrapper {
    fn from(peer: &PeerAddress) -> Self {
        Self {
            host: peer.host.to_string(),
            port: peer.port,
            is_ipv6: peer.is_ipv6,
            display_address: peer.display_address(),
        }
    }
}

// Export discovery result for TypeScript with clean name
#[napi(object, js_name = "DiscoveryResult")]
#[derive(Clone)]
pub struct DiscoveryResultWrapper {
    #[napi(js_name = "ipv4Peers")]
    pub ipv4_peers: Vec<PeerAddressWrapper>,
    #[napi(js_name = "ipv6Peers")]
    pub ipv6_peers: Vec<PeerAddressWrapper>,
    #[napi(js_name = "totalCount")]
    pub total_count: u32,
}

impl From<&DiscoveryResult> for DiscoveryResultWrapper {
    fn from(result: &DiscoveryResult) -> Self {
        Self {
            ipv4_peers: result
                .ipv4_peers
                .iter()
                .map(PeerAddressWrapper::from)
                .collect(),
            ipv6_peers: result
                .ipv6_peers
                .iter()
                .map(PeerAddressWrapper::from)
                .collect(),
            total_count: result.total_count as u32,
        }
    }
}

// Simple address result for single hostname resolution
#[napi(object)]
#[derive(Clone)]
pub struct AddressResult {
    pub addresses: Vec<String>,
    pub count: u32,
}

#[napi]
pub struct DnsDiscoveryClient {
    discovery: DnsDiscovery,
}

#[napi]
impl DnsDiscoveryClient {
    #[napi(constructor)]
    pub fn new() -> Self {
        info!("Creating new DnsDiscoveryClient");
        let rt = tokio::runtime::Handle::current();
        let discovery = rt.block_on(async { DnsDiscovery::new().await }).unwrap();
        Self { discovery }
    }

    #[napi(js_name = "discoverMainnetPeers")]
    pub async fn discover_mainnet_peers(&self) -> Result<DiscoveryResultWrapper> {
        debug!("Discovering mainnet peers");

        let result = self
            .discovery
            .discover_mainnet_peers()
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(DiscoveryResultWrapper::from(&result))
    }

    #[napi(js_name = "discoverTestnet11Peers")]
    pub async fn discover_testnet11_peers(&self) -> Result<DiscoveryResultWrapper> {
        debug!("Discovering testnet11 peers");

        let result = self
            .discovery
            .discover_testnet11_peers()
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(DiscoveryResultWrapper::from(&result))
    }

    #[napi(js_name = "discoverPeers")]
    pub async fn discover_peers(
        &self,
        introducers: Vec<String>,
        default_port: u16,
    ) -> Result<DiscoveryResultWrapper> {
        debug!(
            "Discovering peers using {} introducers with default port {}",
            introducers.len(),
            default_port
        );

        let introducer_refs: Vec<&str> = introducers.iter().map(|s| s.as_str()).collect();
        let result = self
            .discovery
            .discover_peers(&introducer_refs, default_port)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(DiscoveryResultWrapper::from(&result))
    }

    #[napi(js_name = "resolveIpv4")]
    pub async fn resolve_ipv4(&self, hostname: String) -> Result<AddressResult> {
        debug!("Resolving IPv4 addresses for hostname: {}", hostname);

        let addresses = self
            .discovery
            .resolve_ipv4(&hostname)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(AddressResult {
            count: addresses.len() as u32,
            addresses: addresses.into_iter().map(|addr| addr.to_string()).collect(),
        })
    }

    #[napi(js_name = "resolveIpv6")]
    pub async fn resolve_ipv6(&self, hostname: String) -> Result<AddressResult> {
        debug!("Resolving IPv6 addresses for hostname: {}", hostname);

        let addresses = self
            .discovery
            .resolve_ipv6(&hostname)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(AddressResult {
            count: addresses.len() as u32,
            addresses: addresses.into_iter().map(|addr| addr.to_string()).collect(),
        })
    }

    #[napi(js_name = "resolveBoth")]
    pub async fn resolve_both(&self, hostname: String, port: u16) -> Result<DiscoveryResultWrapper> {
        debug!(
            "Resolving both IPv4 and IPv6 addresses for hostname: {} port: {}",
            hostname, port
        );

        let result = self
            .discovery
            .resolve_both(&hostname, port)
            .await
            .map_err(|e| Error::new(Status::GenericFailure, e.to_string()))?;

        Ok(DiscoveryResultWrapper::from(&result))
    }
}

impl Default for DnsDiscoveryClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_client_creation() {
        let client = DnsDiscoveryClient::new();
        assert!(client.is_ok());
    }
}
