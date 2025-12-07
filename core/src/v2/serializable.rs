//! An async compatible implementation of the logic to work with MLS.
//! Avoids the issue of not having an async storage provider API in openmls yet by deserializng and serializing
//! the state before and after every operation to then store with an async storage implementation.
//! Async storage is required as we need to run in the service worker in the browser which only has access to async storage APIs.
//! Running in the service worker allows sending MLS messages through push notifications for example and avoids state management issues
//! with multiple tabs effectively being multiple simultaneous clients that use the same storage.

use std::{collections::HashSet, rc::Rc};

use base64::prelude::*;
use nanoid::nanoid;
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsError, prelude::wasm_bindgen};

use crate::{
    CIPHERSUITE, DecodedPackage, Friend, ID_LENGTH, encode_application_id, v2::provider::Provider,
};

#[derive(Serialize, Deserialize)]
struct User {
    name: Option<String>,
    credential: CredentialWithKey,
    signature_key: SignatureKeyPair,
}

#[derive(Serialize, Deserialize)]
struct Client {
    id: Rc<str>,
    user: User,
    /// We only store the group ids because the groups themselves are not serializable.
    /// The group state can be retrieved from the storage provider using the group id.
    groups: HashSet<GroupId>,

    /// Need to be kept for later reference
    key_packages: Vec<KeyPackage>,
    provider: Provider,
}

#[wasm_bindgen]
pub fn create_client(id: Option<String>, name: Option<String>) -> Result<Vec<u8>, JsError> {
    console_error_panic_hook::set_once();

    let provider = Provider::default();
    let client_id = id.unwrap_or_else(|| nanoid!(ID_LENGTH));

    //TODO Basic credentials only for tests and demo
    let credential: Credential = BasicCredential::new(client_id.clone().into_bytes()).into();
    let signature_keys = SignatureKeyPair::new(CIPHERSUITE.signature_algorithm())?;
    signature_keys.store(provider.storage())?;

    let credential = CredentialWithKey {
        credential,
        signature_key: signature_keys.public().into(),
    };

    let user = User {
        name: name.into(),
        credential,
        signature_key: signature_keys,
    };

    let client = Client {
        id: client_id.into(),
        user,
        groups: HashSet::new(),
        key_packages: Vec::new(),
        provider,
    };

    Ok(postcard::to_allocvec(&client)?)
}

#[wasm_bindgen(getter_with_clone)]
pub struct InviteResult {
    pub client: Box<[u8]>,
    pub invite_payload: String,
}

#[wasm_bindgen]
pub fn create_invite(client: &[u8], user_name: Option<String>) -> Result<InviteResult, JsError> {
    let mut client: Client = postcard::from_bytes(client)?;

    //TODO think about ways to reduce size of key package to generate smaller invite links
    //TODO like using a non self describing serialization format and remove
    //TODO and remove things that do not change or where we use a default
    //TODO adding postcard as dependency yields 9-10% smaller serialized + base64 encoded key packages

    let extensions = encode_application_id(&client.id, &user_name);

    // Add identifier to help users identify the origin of the key package / invitation
    // Details: https://www.rfc-editor.org/rfc/rfc9420.html#section-5.3.3

    let bundle = KeyPackage::builder()
        .key_package_extensions(extensions)
        .build(
            CIPHERSUITE,
            &client.provider,
            &client.user.signature_key,
            client.user.credential.clone(),
        )?;

    client.key_packages.push(bundle.key_package().clone());
    let client = postcard::to_allocvec(&client)?.into();
    // Using postcard reduces the size by around 40 bytes or 9-10%
    // This might not be worth the dependency but we are using it for application messages anyways
    let data = postcard::to_allocvec(bundle.key_package())?;
    Ok(InviteResult {
        client,
        invite_payload: BASE64_URL_SAFE_NO_PAD.encode(data),
    })
}

#[wasm_bindgen]
pub fn decode_key_package(client: &[u8], encoded_invite: &str) -> Result<DecodedPackage, JsError> {
    let data = BASE64_URL_SAFE_NO_PAD.decode(encoded_invite)?;
    // let package = KeyPackageIn::tls_deserialize_exact_bytes(&data).unwrap();
    let package: KeyPackageIn = postcard::from_bytes(&data)?;

    let client: Client = postcard::from_bytes(client)?;

    let validated = package.validate(client.provider.crypto(), ProtocolVersion::Mls10)?;
    let id = validated
        .extensions()
        .application_id()
        .map(|id| str::from_utf8(id.as_slice()))
        .transpose()?
        .ok_or_else(|| {
            JsError::new("Invite did not contain an id to contact the other client with")
        })?;

    let (id, friend_name) = if id.len() > ID_LENGTH {
        let (id, friend_name) = id.split_at(ID_LENGTH);
        (id, Some(friend_name))
    } else {
        (id, None)
    };

    Ok(DecodedPackage {
        friend: Friend {
            id: id.to_owned(),
            name: friend_name.map(str::to_owned),
        },
        key_package: validated,
    })
}
