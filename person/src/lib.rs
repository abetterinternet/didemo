use serde::{Deserialize, Serialize};

/// A person's credential.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PresentedCredential {
    /// The person's name.
    pub name: String,

    /// The person's age in years.
    pub age: u32,
}
