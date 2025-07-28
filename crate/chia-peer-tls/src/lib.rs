//! Shared TLS and peer connection functionality for Chia blockchain clients
//! 
//! This crate provides common TLS certificate handling, WebSocket connector creation,
//! and peer connection utilities that can be shared between ChiaPeerPool and 
//! ChiaBlockListener implementations.

pub mod error;
pub mod tls;
pub mod types;

pub use error::{PeerTlsError, Result};
pub use tls::{
    load_or_generate_cert, 
    create_tls_connector, 
    create_ws_connector,
    calculate_node_id,
    save_cert_to_file,
};
pub use types::{ChiaCertificate, ConnectionInfo, PeerState};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_functionality() {
        // Test that we can generate certificates
        let cert = tls::load_or_generate_cert().unwrap();
        assert!(!cert.private_key.is_empty());
        assert!(!cert.certificate.is_empty());
        assert!(!cert.certificate_der.is_empty());

        // Test that we can create connectors
        let tls_connector = create_tls_connector(&cert).unwrap();
        let ws_connector = create_ws_connector(&cert).unwrap();

        // Test node ID calculation
        let node_id = calculate_node_id(&cert.certificate_der).unwrap();
        assert_eq!(node_id.len(), 64);
    }
} 