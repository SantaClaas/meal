use openmls_rust_crypto::RustCrypto;
use openmls_traits::OpenMlsProvider;
use web_sys::js_sys;

pub(super) mod storage;
pub(super) struct Provider {
    crypto: RustCrypto,
    storage: storage::Provider,
}

impl Provider {
    pub(super) fn new(bridge: js_sys::Function) -> Self {
        Self {
            crypto: RustCrypto::default(),
            storage: storage::Provider::new(bridge),
        }
    }
}

impl OpenMlsProvider for Provider {
    type CryptoProvider = RustCrypto;

    type RandProvider = RustCrypto;

    type StorageProvider = storage::Provider;

    fn storage(&self) -> &Self::StorageProvider {
        &self.storage
    }

    fn crypto(&self) -> &Self::CryptoProvider {
        &self.crypto
    }

    fn rand(&self) -> &Self::RandProvider {
        &self.crypto
    }
}
