use didemo_person::PresentedCredential;
use didemo_wallet::CredentialType;
use std::collections::HashMap;

#[tokio::test]
async fn talk_to_person_and_wallet() {
    // Have the person present their credential
    let presented_credential: PresentedCredential = reqwest::get("http://0.0.0.0:8000/present")
        .await
        .unwrap()
        .json()
        .await
        .unwrap();
    assert_eq!(
        presented_credential,
        PresentedCredential {
            name: "Homer Simpson".to_string(),
            age: 35
        }
    );

    // Dump credentials from the wallet
    let wallet_credentials: HashMap<CredentialType, String> =
        reqwest::get("http://0.0.0.0:8001/credentials")
            .await
            .unwrap()
            .json()
            .await
            .unwrap();
    assert_eq!(
        wallet_credentials,
        HashMap::from([
            (
                CredentialType::LibraryCard,
                r#"{"some": "json"}"#.to_string()
            ),
            (
                CredentialType::DriversLicense,
                r#"{"more": "json"}"#.to_string()
            ),
        ])
    );
}
