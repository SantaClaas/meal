//! An async compatible implementation of the logic to work with MLS.
//! Avoids the issue of not having an async storage provider API in openmls yet by deserializng and serializing
//! the state before and after every operation to then store with an async storage implementation.
//! Async storage is required as we need to run in the service worker in the browser which only has access to async storage APIs.
//! Running in the service worker allows sending MLS messages through push notifications for example and avoids state management issues
//! with multiple tabs effectively being multiple simultaneous clients that use the same storage.

use std::{collections::HashSet, rc::Rc};

use nanoid::nanoid;
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use serde::{Deserialize, Serialize};
use wasm_bindgen::{JsError, prelude::wasm_bindgen};

use crate::{CIPHERSUITE, ID_LENGTH, v2::provider::Provider};

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
    provider: Provider,
}

#[wasm_bindgen]
pub fn create_client(id: Option<String>, name: Option<String>) -> Result<Vec<u8>, JsError> {
    console_error_panic_hook::set_once();

    let provider = Provider::new();
    let client_id = id.unwrap_or_else(|| nanoid!(ID_LENGTH));

    //TODO Basic credentials only for tests and demo
    let credential: Credential = BasicCredential::new(client_id.clone().into_bytes()).into();
    let signature_keys = SignatureKeyPair::new(CIPHERSUITE.signature_algorithm()).unwrap();
    signature_keys.store(provider.storage()).unwrap();

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
        provider,
    };

    postcard::to_allocvec(&client).map_err(JsError::from)
}
