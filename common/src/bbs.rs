//! Utilities for working with BBS signatures and pairing_crypto.

use anyhow::{Context, anyhow};
use pairing_crypto::bbs::{
    BbsProofGenRevealMessageRequest, BbsProofVerifyRequest, BbsSignRequest, BbsVerifyRequest,
    ciphersuites::{
        bls12_381::{BBS_BLS12381G1_SIGNATURE_LENGTH, KeyPair},
        bls12_381_g1_sha_256::{proof_gen, proof_verify, sign, verify},
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
    ) -> Result<(), anyhow::Error> {
        if verify(&BbsVerifyRequest {
            public_key: &self.keypair.public_key.to_octets(),
            header: Some(header),
            messages: Some(&messages),
            signature: &signature_to_array(signature)?,
        })
        .context("failed to verify BBS signature")?
        {
            Ok(())
        } else {
            Err(anyhow!("BBS signature invalid"))
        }
    }

    /// Prove one or more messages from a signature. Messages are tuples; the boolean indicates
    /// whether the message should be revealed with the proof.
    pub fn prove(
        &self,
        header: Vec<u8>,
        messages: Vec<(bool, Vec<u8>)>,
        signature: Vec<u8>,
    ) -> Result<Vec<u8>, anyhow::Error> {
        let proof_gen_reveals: Vec<_> = messages
            .into_iter()
            .map(|(reveal, value)| BbsProofGenRevealMessageRequest { reveal, value })
            .collect();
        proof_gen(&pairing_crypto::bbs::BbsProofGenRequest {
            public_key: &self.keypair.public_key.to_octets(),
            header: Some(header),
            messages: Some(&proof_gen_reveals),
            signature: &signature_to_array(signature)?,
            // TODO: Most definitely want to bind the proof to something like say an authentication
            // session or at least a particular verifier.
            presentation_header: None,
            // why on earth is this an optional boolean? What does None mean that false wouldn't?!
            verify_signature: Some(false),
        })
        .context("failed to BBS prove messages")
    }

    /// Verify one or more messages against a signature
    pub fn verify_proof(
        &self,
        header: Vec<u8>,
        disclosed_messages: Vec<(usize, Vec<u8>)>,
        proof: Vec<u8>,
    ) -> Result<(), anyhow::Error> {
        if proof_verify(&BbsProofVerifyRequest {
            public_key: &self.keypair.public_key.to_octets(),
            header: Some(header),
            // TODO: presentation header binding to context
            presentation_header: None,
            proof: &proof,
            messages: Some(&disclosed_messages),
        })
        .context("failed to verify BBS proofs")?
        {
            Ok(())
        } else {
            Err(anyhow!("BBS proof invalid"))
        }
    }
}

fn signature_to_array(
    signature: Vec<u8>,
) -> Result<[u8; BBS_BLS12381G1_SIGNATURE_LENGTH], anyhow::Error> {
    signature
        .try_into()
        .map_err(|_| anyhow!("failed to convert signature to array"))
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
