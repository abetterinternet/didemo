use serde::{Deserialize, Serialize};

/// Possible types of credentials.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum CredentialType {
    LibraryCard,
    DriversLicense,
}
