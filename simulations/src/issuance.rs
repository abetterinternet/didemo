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

    // Ensure appropriate credentials appear in the wallet
    let mut wallet_credentials: Vec<Credential> = reqwest::get("http://0.0.0.0:8001/credentials")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    wallet_credentials.sort_by_key(|c| c.credential_type);
    let mut expected_wallet_credentials = Vec::from([
        Credential {
            credential_type: CredentialType::DriversLicense,
            encoded_credential: serde_json::to_string(&DriversLicense {
                issuing_jurisdiction: "dmv-1".to_string(),
                holder_name: "Homer Simpson".to_string(),
                serial_number: 1,
                home_address: "742 Evergreen Terrace, Springfield, OH".to_string(),
                organ_donor: true,
                birthdate: 1753729603,
            })
            .unwrap(),
        },
        Credential {
            credential_type: CredentialType::LibraryCard,
            encoded_credential: serde_json::to_string(&LibraryCard {
                library_name: "library-1".to_string(),
                holder_name: "Homer Simpson".to_string(),
                serial_number: 1,
            })
            .unwrap(),
        },
    ]);
    expected_wallet_credentials.sort_by_key(|c| c.credential_type);
    assert_eq!(wallet_credentials, expected_wallet_credentials);

    // TODO: simulate the person visiting a website and proving something to the relying party
}
