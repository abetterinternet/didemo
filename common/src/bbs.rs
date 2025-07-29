//! Utilities for working with BBS signatures and pairing_crypto.

use anyhow::anyhow;
use pairing_crypto::bbs::ciphersuites::bls12_381::KeyPair;

/// Deterministically generate a keypair, diversified using the provided actor_name.
///
/// # Discussion
///
/// Deterministic keys is a cheat since we don't yet have a public key distribution mechanism.
pub fn bbs_keypair(actor_name: &str) -> Result<KeyPair, anyhow::Error> {
    // Using this fixed seed is not secure but this is harmless in the simulation setup.
    KeyPair::new(
        b"00000000000000000000000000000000",
        format!("didemo-{actor_name}").as_bytes(),
    )
    .ok_or(anyhow!("failed to generate BBS key for issuer"))
}

#[cfg(test)]
mod tests {
    use super::bbs_keypair;

    #[test]
    fn keygen_deterministic() {
        let keypair = bbs_keypair("test-1").unwrap();
        let keypair_again = bbs_keypair("test-1").unwrap();

        assert_eq!(keypair, keypair_again);
    }
    #[test]
    fn keygen_differs_by_actor_name() {
        let keypair = bbs_keypair("test-1").unwrap();
        let other_keypair = bbs_keypair("test-2").unwrap();

        assert_ne!(keypair, other_keypair);
    }
}
