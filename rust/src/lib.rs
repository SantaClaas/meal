use std::sync::OnceLock;

use openmls::prelude::*;
use openmls_basic_credential::SignatureKeyPair;
use openmls_rust_crypto::OpenMlsRustCrypto;
use openmls_traits::types::Ciphersuite;
use wasm_bindgen::prelude::*;

pub(crate) const CIPHERSUITE: Ciphersuite =
    Ciphersuite::MLS_128_DHKEMX25519_AES128GCM_SHA256_Ed25519;

struct User {
    credential: CredentialWithKey,
    signature_key: SignatureKeyPair,
}
#[wasm_bindgen]
pub struct AppState {
    user: User,
    groups: Vec<MlsGroup>,
    /// Need to be kept for later reference
    key_packages: Vec<KeyPackage>,
}

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
            credential,
            signature_key: signature_keys,
        };

        Self {
            user,
            groups: Vec::new(),
            key_packages: Vec::new(),
        }
    }

    pub fn generate_key_package(&mut self) -> wrapper::KeyPackage {
        let bundle = KeyPackage::builder()
            .build(
                CIPHERSUITE,
                provider(),
                &self.user.signature_key,
                self.user.credential.clone(),
            )
            .unwrap();

        self.key_packages.push(bundle.key_package().clone());

        wrapper::KeyPackage(bundle.key_package().clone())
    }

    pub fn create_group(&mut self) -> wrapper::GroupId {
        // Does default provide a resonable default configuration?
        // let group_configuration = MlsGroupConfigBuilder::default()
        //     .use_ratchet_tree_extension(true)
        //     .build();

        // let group = MlsGroup::new(
        //     &self.backend,
        //     &self.user.signature_key,
        //     &group_configuration,
        //     self.user.credential.clone(),
        // )
        // .unwrap();

        // let new_index = self.groups.len();
        // self.groups.push(group);
        // new_index
        let group = MlsGroup::builder()
            .use_ratchet_tree_extension(true)
            .build(
                provider(),
                &self.user.signature_key,
                self.user.credential.clone(),
            )
            .unwrap();

        let id = group.group_id().clone();
        self.groups.push(group);
        wrapper::GroupId(id)
    }
}
