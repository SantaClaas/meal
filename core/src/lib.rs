use core::str;
use std::{collections::HashMap, sync::OnceLock};

use base64::prelude::*;
use nanoid::nanoid;
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;
use openmls_traits::types::Ciphersuite;
use tls_codec::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

pub(crate) const CIPHERSUITE: Ciphersuite =
    Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

const ID_LENGTH: usize = 21;

fn provider() -> &'static impl OpenMlsProvider {
    static INSTANCE: OnceLock<OpenMlsRustCrypto> = OnceLock::new();
    INSTANCE.get_or_init(OpenMlsRustCrypto::default)
}

struct User {
    name: Option<String>,
    credential: CredentialWithKey,
    signature_key: SignatureKeyPair,
}

#[wasm_bindgen(getter_with_clone)]
pub struct Client {
    pub id: String,
    user: User,
    groups: HashMap<GroupId, MlsGroup>,
    /// Need to be kept for later reference
    key_packages: Vec<KeyPackage>,
}

#[wasm_bindgen(getter_with_clone)]
pub struct DecodedPackage {
    /// The id of the client that sent the key package
    pub client_id: String,
    pub friend_name: Option<String>,
    key_package: KeyPackage,
}

#[derive(serde::Serialize)]
pub enum Message {
    Private { group_id: String, message: String },
    Welcome { group_id: String },
}

#[wasm_bindgen]
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new(id: Option<String>, name: Option<String>) -> Self {
        console_error_panic_hook::set_once();
        let provider = provider();

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

        Self {
            id: client_id,
            user,
            groups: HashMap::new(),
            key_packages: Vec::new(),
        }
    }

    pub fn get_name(&self) -> Option<String> {
        self.user.name.clone()
    }

    pub fn set_name(&mut self, name: Option<String>) {
        self.user.name = name;
    }

    /// name is name to show on the invite. Does not have to be the same as the name of the user
    pub fn create_invite(&mut self, name: Option<String>) -> String {
        //TODO think about ways to reduce size of key package to generate smaller invite links
        //TODO like using a non self describing serialization format and remove
        //TODO and remove things that do not change or where we use a default
        //TODO adding postcard as dependency yields 9-10% smaller serialized + base64 encoded key packages

        let mut id = self.id.clone();
        if let Some(name) = name {
            id.push_str(&name);
        }

        let extensions = Extensions::single(Extension::ApplicationId(ApplicationIdExtension::new(
            id.as_bytes(),
        )));

        // Add identifier to help users identify the origin of the key package / invitation
        // Details: https://www.rfc-editor.org/rfc/rfc9420.html#section-5.3.3

        let bundle = KeyPackage::builder()
            .key_package_extensions(extensions)
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

        // Need to create id before moving group into map
        let js_group_id = BASE64_URL_SAFE_NO_PAD.encode(group_id.as_slice());
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

    fn process_private_message(&mut self, message: PrivateMessageIn) -> Message {
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

        let message = String::from_utf8(content.into_bytes()).unwrap();
        let js_group_id = BASE64_URL_SAFE_NO_PAD.encode(group.group_id().as_slice());

        Message::Private {
            group_id: js_group_id,
            message,
        }
    }

    fn process_welcome(&mut self, welcome: Welcome) -> Message {
        let provider = provider();
        let configuration = MlsGroupJoinConfig::builder()
            .use_ratchet_tree_extension(true)
            .build();

        let staged =
            StagedWelcome::new_from_welcome(provider, &configuration, welcome, None).unwrap();

        let group = staged.into_group(provider).unwrap();

        let group_id = group.group_id();
        // Need to create id before moving group into map
        let js_group_id = BASE64_URL_SAFE_NO_PAD.encode(group_id.as_slice());

        // self.groups.insert(group_id.clone(), group);
        // js_group_id
        Message::Welcome {
            group_id: js_group_id,
        }
    }

    pub fn process_message(&mut self, message: &[u8]) -> JsValue {
        let message = MlsMessageIn::tls_deserialize_exact_bytes(message).unwrap();

        let value = match message.extract() {
            MlsMessageBodyIn::PrivateMessage(message) => self.process_private_message(message),
            MlsMessageBodyIn::Welcome(welcome) => self.process_welcome(welcome),
            MlsMessageBodyIn::PublicMessage(_public_message_in) => todo!("Public message in"),
            MlsMessageBodyIn::GroupInfo(_verifiable_group_info) => todo!("Group info in"),
            MlsMessageBodyIn::KeyPackage(_key_package_in) => todo!("key package in"),
        };

        serde_wasm_bindgen::to_value(&value).unwrap()
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

    let Some(mut id) = validated.extensions().application_id().map(|id| {
        str::from_utf8(id.as_slice())
            .unwrap_or_else(|_| todo!("Handle id not utf8"))
            .to_owned()
    }) else {
        todo!("No application id provided. Can not contact user")
    };

    let friend_name = if id.len() > ID_LENGTH {
        Some(id.split_off(ID_LENGTH))
    } else {
        None
    };

    DecodedPackage {
        client_id: id,
        friend_name,
        key_package: validated,
    }
}
