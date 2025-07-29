use didemo_common::{
    bbs::bbs_keypair,
    credential::{Credential, CredentialType, DriversLicense, LibraryCard},
    messages::person::ObtainCredentialRequest,
};
use pairing_crypto::bbs::{BbsVerifyRequest, ciphersuites::bls12_381_g1_sha_256::verify};
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
        let (keypair, signature, messages) = match credential.credential_type {
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

                (
                    bbs_keypair("issuer/library-1").unwrap(),
                    credential.signature.clone(),
                    Vec::from([
                        decoded_credential.library_name.into_bytes(),
                        decoded_credential.holder_name.into_bytes(),
                        decoded_credential.serial_number.to_be_bytes().to_vec(),
                    ]),
                )
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

                (
                    bbs_keypair("issuer/dmv-1").unwrap(),
                    credential.signature.clone(),
                    Vec::from([
                        decoded_credential.issuing_jurisdiction.into_bytes(),
                        decoded_credential.holder_name.into_bytes(),
                        decoded_credential.serial_number.to_be_bytes().to_vec(),
                        decoded_credential.home_address.into_bytes(),
                        if decoded_credential.organ_donor {
                            Vec::from([1])
                        } else {
                            Vec::from([0])
                        },
                        decoded_credential.birthdate.to_be_bytes().to_vec(),
                    ]),
                )
            }
        };

        let result = verify(&BbsVerifyRequest {
            public_key: &keypair.public_key.to_octets(),
            header: None,
            messages: Some(&messages),
            signature: &signature.try_into().unwrap(),
        })
        .unwrap();
        assert!(result);
    }

    // TODO: simulate the person visiting a website and proving something to the relying party
}
