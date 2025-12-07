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
use tls_codec::Serialize as _;
use wasm_bindgen::{JsError, prelude::wasm_bindgen};

use crate::{
    ApplicationMessage, CIPHERSUITE, DecodedPackage, Friend, ID_LENGTH, encode_application_id,
    v2::provider::Provider,
};

#[derive(Serialize, Deserialize)]
struct User {
    credential: CredentialWithKey,
    signature_key: SignatureKeyPair,
}

#[wasm_bindgen]
#[derive(Debug, thiserror::Error)]
pub enum CreateGroupError {
    #[error("Group id already exists. This should not happen if the group id is created randomly")]
    IdCollision,
}

#[derive(Debug, thiserror::Error)]
#[error("Group not found")]
pub struct GroupNotFound;

#[derive(Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Client {
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
impl Client {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<Self, JsError> {
        console_error_panic_hook::set_once();

        let provider = Provider::default();
        let client_id = nanoid!(ID_LENGTH);

        //TODO Basic credentials only for tests and demo
        let credential: Credential = BasicCredential::new(client_id.clone().into_bytes()).into();
        let signature_keys = SignatureKeyPair::new(CIPHERSUITE.signature_algorithm())?;
        signature_keys.store(provider.storage())?;

        let credential = CredentialWithKey {
            credential,
            signature_key: signature_keys.public().into(),
        };

        let user = User {
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

        Ok(client)
    }

    pub fn serialize(&self) -> Result<Vec<u8>, JsError> {
        Ok(postcard::to_allocvec(&self)?)
    }

    pub fn from_serialized(bytes: &[u8]) -> Result<Self, JsError> {
        Ok(postcard::from_bytes(bytes)?)
    }

    pub fn create_invite(&mut self, user_name: Option<String>) -> Result<String, JsError> {
        //TODO think about ways to reduce size of key package to generate smaller invite links
        //TODO like using a non self describing serialization format and remove
        //TODO and remove things that do not change or where we use a default
        //TODO adding postcard as dependency yields 9-10% smaller serialized + base64 encoded key packages

        let extensions = encode_application_id(&self.id, &user_name);

        // Add identifier to help users identify the origin of the key package / invitation
        // Details: https://www.rfc-editor.org/rfc/rfc9420.html#section-5.3.3

        let bundle = KeyPackage::builder()
            .key_package_extensions(extensions)
            .build(
                CIPHERSUITE,
                &self.provider,
                &self.user.signature_key,
                self.user.credential.clone(),
            )?;

        self.key_packages.push(bundle.key_package().clone());
        // Using postcard reduces the size by around 40 bytes or 9-10%
        // This might not be worth the dependency but we are using it for application messages anyways
        let data = postcard::to_allocvec(bundle.key_package())?;
        Ok(BASE64_URL_SAFE_NO_PAD.encode(data))
    }

    pub fn decode_key_package(&self, encoded_invite: &str) -> Result<DecodedPackage, JsError> {
        let data = BASE64_URL_SAFE_NO_PAD.decode(encoded_invite)?;
        // let package = KeyPackageIn::tls_deserialize_exact_bytes(&data).unwrap();
        let package: KeyPackageIn = postcard::from_bytes(&data)?;

        let validated = package.validate(self.provider.crypto(), ProtocolVersion::Mls10)?;
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

    pub fn create_group(&mut self) -> Result<String, JsError> {
        let group = MlsGroup::builder()
            .use_ratchet_tree_extension(true)
            // //TODO should we enforce usage of application id in the capabilities?
            // .with_leaf_node_extensions(encode_application_id(self.id.clone(), &self.user.name))
            // .unwrap()
            .build(
                &self.provider,
                &self.user.signature_key,
                self.user.credential.clone(),
            )?;

        let group_id = group.group_id();
        if self.groups.contains(&group_id) {
            return Err(CreateGroupError::IdCollision.into());
        }
        let js_group_id = BASE64_URL_SAFE_NO_PAD.encode(group_id.as_slice());

        self.groups.insert(group_id.clone());
        Ok(js_group_id)
    }

    /// Creates a group using the package decoded in when reading the invite.
    /// Returns the serialized welcome message.
    /// Don't use "package" as a parameter name as it is reserved in JavaScript and will make
    /// the wasm bindgen code fail.
    pub fn invite(
        &mut self,
        group_id: &str,
        key_package: DecodedPackage,
        user_name: Option<String>,
    ) -> Result<Vec<u8>, JsError> {
        let bytes = BASE64_URL_SAFE_NO_PAD.decode(group_id).unwrap();
        let group_id = GroupId::from_slice(&bytes);
        let package = key_package.key_package;
        let storage = self.provider.storage();

        let mut group = MlsGroup::load(storage, &group_id)?.ok_or_else(|| GroupNotFound)?;

        //TODO support multi user groups
        // We don't need the out message because there is no other group members
        // that need to be "informed" of the change (the commit message)
        let (_out_message, welcome, _group_info) =
            group.add_members(&self.provider, &self.user.signature_key, &[package])?;

        // Process it on our end
        group.merge_pending_commit(&self.provider)?;
        // Create introduction message as welcome does not include enough information that are needed on the application layer
        let introduction = ApplicationMessage::Introduction {
            id: self.id.to_string(),
            user_name,
        };

        let data = postcard::to_allocvec(&introduction).unwrap();
        let message = group.create_message(&self.provider, &self.user.signature_key, &data)?;

        //TODO the introduction has to be sent to all group members when we support multi user groups
        // Batch send messages
        //TODO test without vector and U8 slice variant
        let mut vector = Vec::new();
        vector.push(welcome);
        vector.push(message);
        Ok(TlsSliceU8(&vector).tls_serialize_detached()?)
    }
}
