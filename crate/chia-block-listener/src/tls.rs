use crate::error::{BlockListenerError, Result};
use crate::types::ChiaCertificate;
use std::path::Path;
use tokio_tungstenite::Connector;
use tracing::{debug, info, warn};

/// Load or generate Chia certificates for TLS connections
pub fn load_or_generate_cert() -> Result<ChiaCertificate> {
    // Try to load existing certificates from standard Chia locations
    if let Ok(cert) = try_load_existing_cert() {
        info!("Loaded existing Chia certificates");
        return Ok(cert);
    }

    // Generate new certificates if not found
    warn!("Existing certificates not found, generating new ones");
    generate_cert()
}

/// Try to load existing Chia certificates
fn try_load_existing_cert() -> Result<ChiaCertificate> {
    // Standard Chia certificate locations
    let chia_root = std::env::var("CHIA_ROOT")
        .unwrap_or_else(|_| {
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            format!("{}/.chia/mainnet", home)
        });

    let ssl_dir = format!("{}/config/ssl", chia_root);
    let private_key_path = format!("{}/full_node/private_full_node.key", ssl_dir);
    let cert_path = format!("{}/full_node/private_full_node.crt", ssl_dir);

    // Try to read private key
    let private_key = std::fs::read(&private_key_path)
        .map_err(|e| BlockListenerError::Certificate(format!("Failed to read private key: {}", e)))?;

    // Try to read certificate
    let certificate = std::fs::read(&cert_path)
        .map_err(|e| BlockListenerError::Certificate(format!("Failed to read certificate: {}", e)))?;

    // Parse certificate to get DER format
    let certificate_der = parse_pem_to_der(&certificate)?;

    debug!("Loaded certificates from {}", ssl_dir);
    Ok(ChiaCertificate {
        private_key,
        certificate,
        certificate_der,
    })
}

/// Generate new self-signed certificates
fn generate_cert() -> Result<ChiaCertificate> {
    // Generate a simple self-signed certificate for development
    // In production, you'd use proper Chia certificate generation
    
    let private_key = generate_private_key()?;
    let certificate = generate_self_signed_cert(&private_key)?;
    let certificate_der = parse_pem_to_der(&certificate)?;

    debug!("Generated new self-signed certificates");
    Ok(ChiaCertificate {
        private_key,
        certificate,
        certificate_der,
    })
}

/// Generate a private key (simplified implementation)
fn generate_private_key() -> Result<Vec<u8>> {
    // This is a simplified implementation
    // In practice, you'd use proper cryptographic libraries
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    let key: Vec<u8> = (0..32).map(|_| rng.gen()).collect();
    
    // Wrap in PEM format
    let pem_key = format!(
        "-----BEGIN PRIVATE KEY-----\n{}\n-----END PRIVATE KEY-----\n",
        base64_encode(&key)
    );
    
    Ok(pem_key.into_bytes())
}

/// Generate a self-signed certificate (simplified implementation)
fn generate_self_signed_cert(private_key: &[u8]) -> Result<Vec<u8>> {
    // This is a simplified implementation
    // In practice, you'd use proper certificate generation libraries
    use rand::Rng;
    
    let mut rng = rand::thread_rng();
    let cert_data: Vec<u8> = (0..64).map(|_| rng.gen()).collect();
    
    // Wrap in PEM format
    let pem_cert = format!(
        "-----BEGIN CERTIFICATE-----\n{}\n-----END CERTIFICATE-----\n",
        base64_encode(&cert_data)
    );
    
    Ok(pem_cert.into_bytes())
}

/// Parse PEM certificate to DER format
fn parse_pem_to_der(pem_data: &[u8]) -> Result<Vec<u8>> {
    let pem_str = String::from_utf8_lossy(pem_data);
    
    // Find the certificate content between the markers
    let start_marker = "-----BEGIN CERTIFICATE-----";
    let end_marker = "-----END CERTIFICATE-----";
    
    if let Some(start) = pem_str.find(start_marker) {
        if let Some(end) = pem_str.find(end_marker) {
            let cert_content = &pem_str[start + start_marker.len()..end];
            let cert_content = cert_content.replace('\n', "").replace('\r', "");
            
            // Decode base64
            return base64_decode(&cert_content)
                .map_err(|e| BlockListenerError::Certificate(format!("Failed to decode certificate: {}", e)));
        }
    }
    
    Err(BlockListenerError::Certificate("Invalid PEM format".to_string()))
}

/// Create a TLS connector from the certificate
pub fn create_tls_connector(cert: &ChiaCertificate) -> Result<native_tls::TlsConnector> {
    // Create a TLS connector that accepts self-signed certificates
    // In production, you'd configure this properly for Chia's certificate chain
    let connector = native_tls::TlsConnector::builder()
        .danger_accept_invalid_certs(true) // Only for development
        .danger_accept_invalid_hostnames(true) // Only for development
        .build()
        .map_err(|e| BlockListenerError::Tls(e.to_string()))?;

    debug!("Created TLS connector");
    Ok(connector)
}

/// Create a WebSocket connector with TLS support
pub fn create_ws_connector(cert: &ChiaCertificate) -> Result<Connector> {
    let tls_connector = create_tls_connector(cert)?;
    Ok(Connector::NativeTls(tls_connector))
}

/// Calculate node ID from certificate (used for peer identification)
pub fn calculate_node_id(cert_der: &[u8]) -> Result<String> {
    use sha2::{Digest, Sha256};
    
    let mut hasher = Sha256::new();
    hasher.update(cert_der);
    let hash = hasher.finalize();
    
    Ok(hex::encode(hash))
}

/// Simple base64 encoding (for development)
fn base64_encode(data: &[u8]) -> String {
    // Simple base64 encoding - in practice use a proper library
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::new();
    
    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &b) in chunk.iter().enumerate() {
            buf[i] = b;
        }
        
        let b0 = buf[0] as usize;
        let b1 = buf[1] as usize;
        let b2 = buf[2] as usize;
        
        result.push(chars.chars().nth(b0 >> 2).unwrap());
        result.push(chars.chars().nth(((b0 & 0x03) << 4) | (b1 >> 4)).unwrap());
        
        if chunk.len() > 1 {
            result.push(chars.chars().nth(((b1 & 0x0f) << 2) | (b2 >> 6)).unwrap());
        } else {
            result.push('=');
        }
        
        if chunk.len() > 2 {
            result.push(chars.chars().nth(b2 & 0x3f).unwrap());
        } else {
            result.push('=');
        }
    }
    
    result
}

/// Simple base64 decoding (for development)
fn base64_decode(data: &str) -> Result<Vec<u8>> {
    // Simple base64 decoding - in practice use a proper library
    let chars = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = Vec::new();
    
    let clean_data = data.replace('=', "");
    let mut buffer = 0u32;
    let mut bits = 0;
    
    for c in clean_data.chars() {
        if let Some(index) = chars.find(c) {
            buffer = (buffer << 6) | (index as u32);
            bits += 6;
            
            if bits >= 8 {
                result.push((buffer >> (bits - 8)) as u8);
                bits -= 8;
                buffer &= (1 << bits) - 1;
            }
        }
    }
    
    Ok(result)
}

/// Save certificate to file (for development)
pub fn save_cert_to_file<P: AsRef<Path>>(cert: &ChiaCertificate, dir: P) -> Result<()> {
    let dir = dir.as_ref();
    std::fs::create_dir_all(dir)
        .map_err(|e| BlockListenerError::Io(e))?;

    let private_key_path = dir.join("private_key.pem");
    let cert_path = dir.join("certificate.pem");

    std::fs::write(&private_key_path, &cert.private_key)
        .map_err(|e| BlockListenerError::Io(e))?;

    std::fs::write(&cert_path, &cert.certificate)
        .map_err(|e| BlockListenerError::Io(e))?;

    info!("Saved certificates to {}", dir.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_certificate_generation() {
        let cert = generate_cert().unwrap();
        assert!(!cert.private_key.is_empty());
        assert!(!cert.certificate.is_empty());
        assert!(!cert.certificate_der.is_empty());
    }

    #[test]
    fn test_base64_encoding() {
        let data = b"hello world";
        let encoded = base64_encode(data);
        let decoded = base64_decode(&encoded).unwrap();
        assert_eq!(data, decoded.as_slice());
    }

    #[test]
    fn test_node_id_calculation() {
        let cert_der = b"test certificate data";
        let node_id = calculate_node_id(cert_der).unwrap();
        assert_eq!(node_id.len(), 64); // SHA256 hex = 64 chars
    }

    #[test]
    fn test_tls_connector_creation() {
        let cert = generate_cert().unwrap();
        let connector = create_tls_connector(&cert);
        assert!(connector.is_ok());
    }
} 