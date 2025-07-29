//! Message definitions for RPCs exchanged between protocol actors.

/// API objects for interacting with an issuer.
pub mod issuer {
    use crate::credential::CredentialType;
    use serde::{Deserialize, Serialize};

    /// A request for the issuer to issue a credential.
    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub struct IssueCredentialRequest {
        /// The type of credential being issued.
        pub credential_type: CredentialType,

        /// A JSON blob, whose format is dictated by `credential_type`, describing the credential being
        /// requested.
        pub requested_credential: String,

        /// The wallet into which the issued credential should be programmed. A DNS name resolvable
        /// by the issuer receiving this request.
        pub wallet_hostname: String,
    }
}

/// API objects for interacting with a person.
pub mod person {
    use crate::credential::CredentialType;
    use serde::{Deserialize, Serialize};

    /// A request for the person to prove a message.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct ProofRequest {
        /// The type of proof requested.
        pub proof_type: ProofType,
        // TODO: should there be a way for the verifier to indicate what issuers it trusts?
        // TODO: some parameters here that get folded into the presentation header?
    }

    /// A type of proof.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub enum ProofType {
        /// Proof that the holder holds a driver's license.
        HoldsDriversLicense,
        /// Proof that the holder holds a library card.
        HoldsLibraryCard,
        /// Proof of the holder's name (discloses a name message to verifier).
        HolderName,
    }

    /// A proof of some message, corresponding to a ProofRequest.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct Proof {
        /// The header from the BBS signature.
        pub header: Vec<u8>,

        /// The BBS proof.
        pub proof: Vec<u8>,

        /// Messages disclosed in the proof. Tuple of message index and message.
        pub disclosed_messages: Vec<(usize, Vec<u8>)>,
    }

    /// A request for a person to obtain a credential.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct ObtainCredentialRequest {
        /// The credential type.
        pub credential_type: CredentialType,

        /// The issuer to obtain the credential from, as a DNS name that this actor can resolve.
        pub issuer: String,
    }
}
