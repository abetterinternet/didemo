//! Utilities for working with BBS signatures and pairing_crypto.

use anyhow::{Context, anyhow};
use pairing_crypto::bbs::{
    BbsSignRequest, BbsVerifyRequest,
    ciphersuites::{
        bls12_381::KeyPair,
        bls12_381_g1_sha_256::{sign, verify},
    },
};

/// A BBS keypair used for signing credentials.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BbsKeypair {
    /// Name of the actor that holds the private portion of this keypair. Used to derive keys.
    actor_name: String,

    /// The keypair.
    keypair: KeyPair,
}

impl BbsKeypair {
    /// Deterministically generate a keypair, diversified using the provided actor_name.
    ///
    /// # Discussion
    ///
    /// Deterministic keys is a cheat since we don't yet have a public key distribution mechanism.
    pub fn new(actor_name: &str) -> Result<Self, anyhow::Error> {
        Ok(Self {
            actor_name: actor_name.to_string(),
            // Using this fixed seed is not secure but this is harmless in the simulation setup.
            keypair: KeyPair::new(
                b"00000000000000000000000000000000",
                format!("didemo-{actor_name}").as_bytes(),
            )
            .ok_or(anyhow!("failed to generate BBS key for issuer"))?,
        })
    }

    /// Sign a message with a header with this key.
    // TODO: take header and messages as references to slices.
    pub fn sign(&self, header: Vec<u8>, messages: Vec<Vec<u8>>) -> Result<Vec<u8>, anyhow::Error> {
        sign(&BbsSignRequest {
            secret_key: &self.keypair.secret_key.to_bytes(),
            public_key: &self.keypair.public_key.to_octets(),
            header: Some(header),
            messages: Some(&messages),
        })
        .map(|s| s.to_vec())
        .context("failed to sign messages")
    }

    /// Verify a signature over a message and header using this key.
    // TODO: take arguments as slices.
    pub fn verify(
        &self,
        header: Vec<u8>,
        messages: Vec<Vec<u8>>,
        signature: Vec<u8>,
    ) -> Result<bool, anyhow::Error> {
        verify(&BbsVerifyRequest {
            public_key: &self.keypair.public_key.to_octets(),
            header: Some(header),
            messages: Some(&messages),
            signature: &signature
                .try_into()
                .map_err(|_| anyhow!("failed to convert signature to array"))?,
        })
        .context("failed to verify BBS signature")
    }
}

#[cfg(test)]
mod tests {
    use super::BbsKeypair;

    #[test]
    fn keygen_deterministic() {
        let keypair = BbsKeypair::new("test-1").unwrap();
        let keypair_again = BbsKeypair::new("test-1").unwrap();

        assert_eq!(keypair, keypair_again);
    }
    #[test]
    fn keygen_differs_by_actor_name() {
        let keypair = BbsKeypair::new("test-1").unwrap();
        let other_keypair = BbsKeypair::new("test-2").unwrap();

        assert_ne!(keypair, other_keypair);
    }
}
