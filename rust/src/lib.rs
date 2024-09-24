use std::{collections::HashMap, rc::Rc, sync::OnceLock};

use base64::prelude::*;
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;
use openmls_traits::types::Ciphersuite;
use tls_codec::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub(crate) const CIPHERSUITE: Ciphersuite =
    Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

fn provider() -> &'static impl OpenMlsProvider {
    static INSTANCE: OnceLock<OpenMlsRustCrypto> = OnceLock::new();
    INSTANCE.get_or_init(OpenMlsRustCrypto::default)
}

/// Wrappers to make use with wasm bindgen easier
mod wrapper {
    use super::*;
    #[wasm_bindgen]
    pub struct GroupId(pub(super) openmls::group::GroupId);

    #[wasm_bindgen]
    pub struct KeyPackage(pub(super) openmls::key_packages::KeyPackage);
}

struct User {
    name: String,
    credential: CredentialWithKey,
    signature_key: SignatureKeyPair,
}

#[wasm_bindgen]
pub struct AppState {
    user: User,
    groups: HashMap<GroupId, MlsGroup>,
    /// Need to be kept for later reference
    key_packages: Vec<KeyPackage>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct Invite {
    pub group_id: String,
    pub welcome: String,
}

#[wasm_bindgen]
impl AppState {
    #[wasm_bindgen(constructor)]
    pub fn new(name: &str) -> Self {
        console_error_panic_hook::set_once();
        let provider = provider();

        //TODO Basic credentials only for tests and demo
        let credential: Credential = BasicCredential::new(name.into()).into();

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

        Self {
            user,
            groups: HashMap::new(),
            key_packages: Vec::new(),
        }
    }

    pub fn get_name(&self) -> String {
        self.user.name.clone().into()
    }

    pub fn generate_key_package(&mut self) -> String {
        let bundle = KeyPackage::builder()
            .build(
                CIPHERSUITE,
                provider(),
                &self.user.signature_key,
                self.user.credential.clone(),
            )
            .unwrap();

        self.key_packages.push(bundle.key_package().clone());

        let data = bundle.key_package().tls_serialize_detached().unwrap();
        let base64 = BASE64_URL_SAFE_NO_PAD.encode(data);
        base64
    }

    pub fn establish_contact(&mut self, encoded_package: String) -> Invite {
        // Decode and verify package from other person
        let data = BASE64_URL_SAFE_NO_PAD.decode(encoded_package).unwrap();
        let package = KeyPackageIn::tls_deserialize_exact_bytes(&data).unwrap();
        let provider = provider();
        let package = package
            .validate(provider.crypto(), ProtocolVersion::Mls10)
            .unwrap();

        // Create group that is the one to one chat
        let mut group = MlsGroup::builder()
            .use_ratchet_tree_extension(true)
            .build(
                provider,
                &self.user.signature_key,
                self.user.credential.clone(),
            )
            .unwrap();

        let (_out_message, welcome, _group_info) = group
            .add_members(provider, &self.user.signature_key, &[package])
            .unwrap();

        // Don't need to send the mls commit message
        // because there are no other group members

        // Process the message
        group.merge_pending_commit(provider).unwrap();
        // Group id to receive messages
        let group_id = group.group_id().clone();
        self.groups.insert(group_id.clone(), group);

        let group_id = BASE64_URL_SAFE_NO_PAD.encode(group_id.as_slice());

        // Send welcome
        let data = welcome.tls_serialize_detached().unwrap();
        let encoded = BASE64_URL_SAFE_NO_PAD.encode(data);
        Invite {
            group_id,
            welcome: encoded,
        }
    }

    //TODO use Box<[u8]> as return type to get Uint8Array in JS
    //TODO accept &[u8] from Uint8Array in JS
    pub fn join_group(&mut self, encoded_welcome: String) -> String {
        let data = BASE64_URL_SAFE_NO_PAD.decode(encoded_welcome).unwrap();
        let message = MlsMessageIn::tls_deserialize_exact_bytes(&data).unwrap();
        let MlsMessageBodyIn::Welcome(welcome) = message.extract() else {
            panic!("Did not expect non welcome");
        };

        let provider = provider();
        // Create group
        let configuration = MlsGroupJoinConfig::builder()
            .use_ratchet_tree_extension(true)
            .build();

        let group = StagedWelcome::new_from_welcome(provider, &configuration, welcome, None)
            .unwrap()
            .into_group(provider)
            .unwrap();

        let encoded = BASE64_URL_SAFE_NO_PAD.encode(group.group_id().as_slice());

        self.groups.insert(group.group_id().clone(), group);
        encoded
    }

    pub fn send_message(&mut self, group_id: String, message: String) -> String {
        let bytes = BASE64_URL_SAFE_NO_PAD.decode(group_id).unwrap();
        let id = GroupId::from_slice(&bytes);
        let group = self.groups.get_mut(&id).unwrap();
        let provider = provider();
        let message = group
            .create_message(provider, &self.user.signature_key, message.as_bytes())
            .unwrap();

        let serialized = message.tls_serialize_detached().unwrap();
        let encoded = BASE64_URL_SAFE_NO_PAD.encode(serialized);
        encoded
    }

    pub fn receive_message(&mut self, group_id: String, message: String) -> String {
        let bytes = BASE64_URL_SAFE_NO_PAD.decode(group_id).unwrap();
        let id = GroupId::from_slice(&bytes);
        let group = self.groups.get_mut(&id).unwrap();
        let provider = provider();
        let bytes = BASE64_URL_SAFE_NO_PAD.decode(message).unwrap();
        let message = MlsMessageIn::tls_deserialize_exact(&bytes).unwrap();
        let MlsMessageBodyIn::PrivateMessage(message) = message.extract() else {
            todo!()
        };

        // let message: ProtocolMessage = message.into();
        let message = group.process_message(provider, message).unwrap();
        let ProcessedMessageContent::ApplicationMessage(message) = message.into_content() else {
            todo!()
        };

        String::from_utf8(message.into_bytes()).unwrap()
    }
}
