use core::str;
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
pub struct DecodedPackage {
    pub friend_name: Option<String>,
    key_package: KeyPackage,
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

    /// name is name to show on the invite. Does not have to be the same as the name of the user
    pub fn create_invite(&mut self, name: Option<String>) -> String {
        //TODO think about ways to reduce size of key package to generate smaller invite links
        //TODO like using a non self describing serialization format and remove
        //TODO and remove things that do not change or where we use a default
        //TODO adding postcard as dependency yields 9-10% smaller serialized + base64 encoded key packages

        // Add identifier to help users identify the origin of the key package / invitation
        // Details: https://www.rfc-editor.org/rfc/rfc9420.html#section-5.3.3
        let builder = KeyPackage::builder();

        let builder = if let Some(name) = name {
            let id = ApplicationIdExtension::new(name.as_bytes());
            builder.key_package_extensions(Extensions::single(Extension::ApplicationId(id)))
        } else {
            builder
        };

        let bundle = builder
            .build(
                CIPHERSUITE,
                provider(),
                &self.user.signature_key,
                self.user.credential.clone(),
            )
            .unwrap();

        self.key_packages.push(bundle.key_package().clone());

        let data = bundle.key_package().tls_serialize_detached().unwrap();
        BASE64_URL_SAFE_NO_PAD.encode(data)
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

    pub fn create_group(&mut self) -> String {
        let provider = provider();
        let group = MlsGroup::builder()
            .use_ratchet_tree_extension(true)
            .build(
                provider,
                &self.user.signature_key,
                self.user.credential.clone(),
            )
            .unwrap();

        let group_id = group.group_id();

        if self.groups.contains_key(&group_id) {
            todo!("Group id collision that should not happen if group id is random");
        }

        let js_group_id = BASE64_URL_SAFE_NO_PAD.encode(group_id.as_slice());
        // Need to create id before moving group into map
        self.groups.insert(group_id.clone(), group);
        js_group_id
    }

    /// Creates a group using the package decoded in when reading the invite.
    /// Returns the serialized welcome message.
    /// Don't use "package" as a paramter name as it is reserved in JavaScript and will make
    /// the wasm bindgen code fail.
    pub fn invite(&mut self, group_id: &str, key_package: DecodedPackage) -> Vec<u8> {
        let bytes = BASE64_URL_SAFE_NO_PAD.decode(group_id).unwrap();
        let group_id = GroupId::from_slice(&bytes);
        let package = key_package.key_package;
        let provider = provider();
        let Some(group) = self.groups.get_mut(&group_id) else {
            todo!("Group does not exist");
        };

        //TODO support multi user groups
        // We don't need the out message bedcause there is no other group members
        // that need to be "informed" of the change (the commit message)
        let (_out_message, welcome, _group_info) = group
            .add_members(provider, &self.user.signature_key, &[package])
            .unwrap();

        // Process it on our end
        group.merge_pending_commit(provider).unwrap();

        welcome.tls_serialize_detached().unwrap()
    }

    pub fn send_message(&mut self, group_id: String, message: String) -> Box<[u8]> {
        let bytes = BASE64_URL_SAFE_NO_PAD.decode(group_id).unwrap();
        let id = GroupId::from_slice(&bytes);
        let group = self.groups.get_mut(&id).unwrap();
        let provider = provider();
        let message = group
            .create_message(provider, &self.user.signature_key, message.as_bytes())
            .unwrap();

        let serialized = message.tls_serialize_detached().unwrap();
        serialized.into_boxed_slice()
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

    fn process_private_message(&mut self, message: PrivateMessageIn) -> String {
        let message = ProtocolMessage::from(message);
        let Some(group) = self.groups.get_mut(message.group_id()) else {
            todo!("Group does not exist");
        };

        let Ok(message) = group.process_message(provider(), message) else {
            todo!("Message processing error");
        };

        let ProcessedMessageContent::ApplicationMessage(content) = message.into_content() else {
            todo!("Handle processed message content");
        };

        String::from_utf8(content.into_bytes()).unwrap()
    }

    pub fn process_message(&mut self, message: &[u8]) -> String {
        let message = MlsMessageIn::tls_deserialize_exact_bytes(message).unwrap();
        match message.extract() {
            MlsMessageBodyIn::PrivateMessage(message) => self.process_private_message(message),
            MlsMessageBodyIn::PublicMessage(public_message_in) => todo!("Public message in"),
            MlsMessageBodyIn::Welcome(welcome) => todo!("Welcome message in"),
            MlsMessageBodyIn::GroupInfo(verifiable_group_info) => todo!("Group info in"),
            MlsMessageBodyIn::KeyPackage(key_package_in) => todo!("key package in"),
        }
    }
}

#[wasm_bindgen]
pub fn decode_key_package(encoded: &str) -> DecodedPackage {
    let data = BASE64_URL_SAFE_NO_PAD.decode(encoded).unwrap();
    let package = KeyPackageIn::tls_deserialize_exact_bytes(&data).unwrap();
    let provider = provider();

    let validated = package
        .validate(provider.crypto(), ProtocolVersion::Mls10)
        .unwrap();

    let id = validated.extensions().application_id().map(|id| {
        str::from_utf8(id.as_slice())
            .unwrap_or_else(|_| todo!("Handle id not utf8"))
            .to_owned()
    });

    DecodedPackage {
        friend_name: id,
        key_package: validated,
    }
}
