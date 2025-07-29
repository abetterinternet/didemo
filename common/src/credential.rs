//! Common definitions for representing and working with credentials.

use serde::{Deserialize, Serialize};

/// Possible types of credentials.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash, Ord, PartialOrd)]
pub enum CredentialType {
    LibraryCard,
    DriversLicense,
}

/// An issued credential.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub struct Credential {
    /// The type of the credential.
    pub credential_type: CredentialType,
    /// Opaque JSON encoding of the credential. Can be decoded based on the value of
    /// `credential_type`.
    pub encoded_credential: String,
    /// BBS signature over the messages making up this credential.
    pub signature: Vec<u8>,
}

impl Credential {
    pub fn bbs_messages(&self) -> Vec<Vec<u8>> {
        todo!("construct vector of BBS messages to sign, verify or prove for this credential")
    }
}

/// A library card.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct LibraryCard {
    pub library_name: String,
    pub holder_name: String,
    pub serial_number: u64,
}

/// A request for a library card.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct LibraryCardRequest {
    pub holder_name: String,
}

/// A driver's license.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct DriversLicense {
    pub issuing_jurisdiction: String,
    pub holder_name: String,
    pub serial_number: u64,
    pub home_address: String,
    pub organ_donor: bool,
    // Holder's birthdate, in seconds since the UNIX epoch.
    pub birthdate: u64,
}

/// A request for a driver's license.
#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct DriversLicenseRequest {
    pub holder_name: String,
    pub home_address: String,
    pub organ_donor: bool,
    pub birthdate: u64,
}
