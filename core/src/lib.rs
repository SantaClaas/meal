use core::str;
use std::{collections::HashMap, sync::OnceLock, vec::IntoIter};

use base64::prelude::*;
use nanoid::nanoid;
use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;
use openmls_traits::types::Ciphersuite;
use serde::{Deserialize, Serialize};
use tls_codec::{Deserialize as _, Serialize as _};
use wasm_bindgen::prelude::*;

pub(crate) const CIPHERSUITE: Ciphersuite =
    Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

/// Shared variable to use for encoding and decoding id
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
    /// The friend that sent the key package
    pub friend: Friend,
    key_package: KeyPackage,
}

#[derive(serde::Serialize, Clone)]
#[wasm_bindgen(getter_with_clone)]
pub struct Friend {
    pub id: String,
    pub name: Option<String>,
}

#[derive(serde::Serialize)]
#[serde(tag = "type")]
enum Message {
    Private { group_id: String, message: String },
    Welcome { group_id: String, friend: Friend },
}

/// The result of processing a welcome message and the introduction message
#[deprecated = "Use GroupInvite instead"]
#[derive(Serialize)]
struct GroupInvite {
    /// The id of the group that was created from the welcome message
    group_id: String,
    /// The id of the client that sent the welcome message
    client_id: String,
    /// The name of the client that sent the welcome message
    name: Option<String>,
}

fn encode_application_id(mut id: String, name: &Option<String>) -> Extensions {
    if let Some(name) = name {
        id.push_str(name);
    }

    Extensions::single(Extension::ApplicationId(ApplicationIdExtension::new(
        id.as_bytes(),
    )))
}

fn decode_application_id(application_id: &ApplicationIdExtension) -> (String, Option<String>) {
    let string = str::from_utf8(application_id.as_slice()).unwrap();
    if string.len() > ID_LENGTH {
        (
            string[..ID_LENGTH].to_owned(),
            Some(string[ID_LENGTH..].to_owned()),
        )
    } else {
        (string.to_owned(), None)
    }
}

/// Defines a message sent on the application layer.
/// This gets transported using MLS but is otherwise independent of the protocol.
#[derive(Serialize, Deserialize)]
enum ApplicationMessage {
    //TODO ask if there is really no possibility to include information in the welcome
    /// This message is sent as a follow up to a welcome message to introduce the sender of the welcome message to
    /// the receiver. Otherwise the client would not know who send the welcome as that information can not be included
    /// in the welcome message (AFAIK). The client needs to know who sent the welcome to be able to know where to send
    /// responses and other messages back to.
    /// <details>
    /// <summary>Discussion</summary>
    /// The delivery service could include the sender information as it knows at what endpoint the message is received
    /// and only clients authorized to send messages should be able to send messages. But we want the delivery service
    /// to be involved as little as possible. This also makes the implementation less reliant on the delivery service.
    ///
    /// It is also not possible to tie the welcome to the introduction, because nesting the welcome in the introduction
    /// would lead to them arriving out of order and it seems that the application message is not decryptable when the
    /// welcome hasn't been processed yet.
    /// </details>
    Introduction {
        /// The client id of the sender that send the welcome message
        id: String,
        /// The name of the user using the client that send the welcome message
        name: Option<String>,
    },
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

        let extensions = encode_application_id(self.id.clone(), &name);

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

        // Using postcard reduces the size by around 40 bytes or 9-10%
        // This might not be worth the dependency but we are using it for application messages anyways
        let data = postcard::to_allocvec(bundle.key_package()).unwrap();
        BASE64_URL_SAFE_NO_PAD.encode(data)
    }

    #[deprecated = "Use process_welcome instead"]
    pub fn join_group(&mut self, encoded_welcome: String) -> JsValue {
        let data = BASE64_URL_SAFE_NO_PAD.decode(encoded_welcome).unwrap();
        let messages = TlsVecU8::<MlsMessageIn>::tls_deserialize_exact_bytes(&data)
            .unwrap()
            .into_vec();

        let (welcome, introduction) = {
            let mut iterator = messages.into_iter();
            (iterator.next().unwrap(), iterator.next().unwrap())
        };

        // Step 1: Process welcome
        let MlsMessageBodyIn::Welcome(welcome) = welcome.extract() else {
            panic!("Did not expect non welcome");
        };

        let provider = provider();
        // Create group
        let configuration = MlsGroupJoinConfig::builder()
            .use_ratchet_tree_extension(true)
            .build();

        let mut group = StagedWelcome::new_from_welcome(provider, &configuration, welcome, None)
            .unwrap()
            .into_group(provider)
            .unwrap();

        // Step 2: Process introduction
        let MlsMessageBodyIn::PrivateMessage(introduction) = introduction.extract() else {
            todo!("Did not expect non application message");
        };

        // Extract introduction application message with new group now
        let introduction = group.process_message(provider, introduction).unwrap();
        let ProcessedMessageContent::ApplicationMessage(content) = introduction.into_content()
        else {
            todo!("Handle processed message content");
        };

        let ApplicationMessage::Introduction { id, name } =
            postcard::from_bytes(&content.into_bytes()).unwrap();
        // Extract id before moving group into map
        let js_group_id = BASE64_URL_SAFE_NO_PAD.encode(group.group_id().as_slice());
        // Add group after introduction has been processed
        self.groups.insert(group.group_id().clone(), group);

        serde_wasm_bindgen::to_value(&GroupInvite {
            group_id: js_group_id,
            client_id: id,
            name,
        })
        .unwrap()
    }

    pub fn create_group(&mut self) -> String {
        let provider = provider();
        let group = MlsGroup::builder()
            .use_ratchet_tree_extension(true)
            // //TODO should we enforce usage of application id in the capabilities?
            // .with_leaf_node_extensions(encode_application_id(self.id.clone(), &self.user.name))
            // .unwrap()
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

        // Create introduction message as welcome does not include enough information that are needed on the application layer
        let introduction = ApplicationMessage::Introduction {
            id: self.id.clone(),
            name: self.user.name.clone(),
        };
        let data = postcard::to_allocvec(&introduction).unwrap();
        let message = group
            .create_message(provider, &self.user.signature_key, &data)
            .unwrap();

        //TODO the introduction has to be sent to all group members when we support multi user groups
        // Batch send messages
        //TODO test without vector and U8 slice variant
        let mut vector = Vec::new();
        vector.push(welcome);
        vector.push(message);
        TlsSliceU16(&vector).tls_serialize_detached().unwrap()
    }

    pub fn send_message(&mut self, group_id: String, message: String) -> Box<[u8]> {
        let bytes = BASE64_URL_SAFE_NO_PAD.decode(group_id).unwrap();
        let id = GroupId::from_slice(&bytes);
        let group = self.groups.get_mut(&id).unwrap();
        let provider = provider();
        let message = group
            .create_message(provider, &self.user.signature_key, message.as_bytes())
            .unwrap();

        // We can batch send messages so we need to wrap it in a collection
        let messages = &[message];
        let message = TlsSliceU16(messages);

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

    fn process_welcome(&mut self, welcome: Welcome, mut rest: IntoIter<MlsMessageIn>) -> Message {
        let introduction = rest.next().unwrap();

        // Step 1: Process welcome
        let provider = provider();
        let configuration = MlsGroupJoinConfig::builder()
            .use_ratchet_tree_extension(true)
            .build();

        let mut group = StagedWelcome::new_from_welcome(provider, &configuration, welcome, None)
            .unwrap()
            .into_group(provider)
            .unwrap();

        // Step 2: Process introduction
        let MlsMessageBodyIn::PrivateMessage(introduction) = introduction.extract() else {
            todo!("Did not expect non application message");
        };

        // Extract introduction application message with new group now
        let introduction = group.process_message(provider, introduction).unwrap();
        let ProcessedMessageContent::ApplicationMessage(content) = introduction.into_content()
        else {
            todo!("Handle processed message content");
        };

        let ApplicationMessage::Introduction { id, name } =
            postcard::from_bytes(&content.into_bytes()).unwrap();

        // Need to create id before moving group into map
        let js_group_id = BASE64_URL_SAFE_NO_PAD.encode(group.group_id().as_slice());
        // Add group after introduction has been processed
        self.groups.insert(group.group_id().clone(), group);

        Message::Welcome {
            friend: Friend { id, name },
            group_id: js_group_id,
        }
    }

    pub fn process_message(&mut self, data: &[u8]) -> JsValue {
        let mut messages = TlsVecU16::<MlsMessageIn>::tls_deserialize_exact_bytes(data)
            .unwrap()
            .into_vec()
            .into_iter();

        // let message = MlsMessageIn::tls_deserialize_exact_bytes(data).unwrap();
        let message = messages.next().unwrap();
        let value = match message.extract() {
            MlsMessageBodyIn::PrivateMessage(message) => self.process_private_message(message),
            MlsMessageBodyIn::Welcome(welcome) => self.process_welcome(welcome, messages),
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
    // let package = KeyPackageIn::tls_deserialize_exact_bytes(&data).unwrap();
    let package: KeyPackageIn = postcard::from_bytes(&data).expect("Expected valid key package");
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
        friend: Friend {
            id,
            name: friend_name,
        },
        key_package: validated,
    }
}
