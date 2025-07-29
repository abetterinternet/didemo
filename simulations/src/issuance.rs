use didemo_common::{
    credential::{Credential, CredentialType, DriversLicense, LibraryCard},
    messages::person::ObtainCredentialRequest,
};
use reqwest::StatusCode;

#[tokio::test]
async fn issue_credential() {
    let client = reqwest::Client::new();

    // Person obtains a driver's license
    let obtain_drivers_request = ObtainCredentialRequest {
        credential_type: CredentialType::DriversLicense,
        issuer: "issuer-dmv".to_string(),
    };
    let response = client
        .post("http://0.0.0.0:8000/credential")
        .json(&obtain_drivers_request)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Person obtains a library card
    let obtain_library_card_request = ObtainCredentialRequest {
        credential_type: CredentialType::LibraryCard,
        issuer: "issuer-library".to_string(),
    };
    let response = client
        .post("http://0.0.0.0:8000/credential")
        .json(&obtain_library_card_request)
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), StatusCode::CREATED);

    // Ensure appropriate credentials appear in the wallet and that the signatures verify. Note that
    // we aren't yet doing any privacy preserving proof stuff. Signature verification reveals all
    // the messages to the verifier.
    let mut wallet_credentials: Vec<Credential> = reqwest::get("http://0.0.0.0:8001/credentials")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    wallet_credentials.sort_by_key(|c| c.credential_type);

    let mut saw_drivers = false;
    let mut saw_library = false;
    for credential in &wallet_credentials {
        match credential.credential_type {
            CredentialType::LibraryCard => {
                assert!(!saw_library, "multiple library cards present");
                saw_library = true;
                let decoded_credential: LibraryCard =
                    serde_json::from_str(&credential.encoded_credential).unwrap();
                assert_eq!(
                    decoded_credential,
                    LibraryCard {
                        library_name: "library-1".to_string(),
                        holder_name: "Homer Simpson".to_string(),
                        serial_number: 1,
                    },
                );
            }
            CredentialType::DriversLicense => {
                assert!(!saw_drivers, "multiple drivers licenses present");
                saw_drivers = true;
                let decoded_credential: DriversLicense =
                    serde_json::from_str(&credential.encoded_credential).unwrap();
                assert_eq!(
                    decoded_credential,
                    DriversLicense {
                        issuing_jurisdiction: "dmv-1".to_string(),
                        holder_name: "Homer Simpson".to_string(),
                        serial_number: 1,
                        home_address: "742 Evergreen Terrace, Springfield, OH".to_string(),
                        organ_donor: true,
                        birthdate: 1753729603,
                    }
                );
            }
        };
    }

    // TODO: simulate the person visiting a website and proving something to the relying party
}
