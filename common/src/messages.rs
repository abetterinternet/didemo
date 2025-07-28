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

    /// A person's credential.
    #[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
    pub struct PresentedCredential {
        /// The person's name.
        pub name: String,

        /// The person's age in seconds since the Unix epoch.
        pub birthdate: u64,
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
