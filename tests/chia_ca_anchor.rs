//! Pins the Chia CA certificate this crate ships as its trust anchor.
//!
//! WHY: `CHIA_CA_CRT` is the root `create_tls_connector` adds when dialling a Chia peer. It arrives
//! as `include_str!` of a file inside `chia-ssl`, so its value changes silently with a dependency
//! bump — no call site changes, nothing fails to compile, and no other test in this crate reads it.
//! A trust anchor that can be swapped without anything going red is exactly the constant worth
//! pinning: this test turns an invisible substitution into a red build that a human must approve.
//!
//! Upstream RE-ISSUED this CA; it did NOT rotate the key. chia-ssl 0.36.1 shipped the 2021
//! certificate (valid to 2031), 0.42.1 ships a re-issue generated 2025-11-19 and valid to 2037.
//! Decoded, the two certificates carry an **identical SPKI, RSA modulus, serial and subject**
//! (`O=Chia, CN=Chia CA, OU=Organic Farming Division`); only the validity dates, the SKI/AKI
//! extensions and therefore the fingerprint differ.
//!
//! That distinction is the whole point of reading this comment: a re-issue of the same keypair
//! introduces no new authority to vet and cannot split interop with peers still on the old cert,
//! whereas a genuine key rotation would do both. Stating it as a rotation would be a false security
//! claim committed to the repo, which is worse than saying nothing.
//!
//! The digest pin below is still worth having, and for an unchanged reason: it makes ANY future
//! substitution — re-issue or true rotation — go red and require a human to look at which one it is.

use chia_ssl::CHIA_CA_CRT;
use sha2::{Digest, Sha256};

/// The SHA-256 of the CA certificate PEM shipped by chia-ssl 0.42.1.
const EXPECTED_CA_PEM_SHA256: &str =
    "86b185d6059ba9b0f2e68080b39ab7fd8bcab39c590917f93aa9fcf6ad260351";

#[test]
fn shipped_chia_ca_is_the_expected_trust_anchor() {
    let digest = Sha256::digest(CHIA_CA_CRT.as_bytes());
    assert_eq!(
        hex_lower(&digest),
        EXPECTED_CA_PEM_SHA256,
        "the Chia CA trust anchor changed. This is a security-relevant substitution: verify the new \
         certificate is a legitimate upstream rotation before updating this digest.",
    );
}

#[test]
fn shipped_chia_ca_is_a_single_well_formed_pem_certificate() {
    // Guards the digest above against being satisfied by something that is not a certificate at
    // all, and against a bundle quietly gaining a SECOND root — one extra anchor is one extra
    // issuer that can mint a peer identity this crate would accept.
    assert!(CHIA_CA_CRT.starts_with("-----BEGIN CERTIFICATE-----"));
    assert!(CHIA_CA_CRT
        .trim_end()
        .ends_with("-----END CERTIFICATE-----"));
    assert_eq!(
        CHIA_CA_CRT.matches("-----BEGIN CERTIFICATE-----").count(),
        1
    );
}

fn hex_lower(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}
