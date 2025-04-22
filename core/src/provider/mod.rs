use openmls_rust_crypto::RustCrypto;
use openmls_traits::OpenMlsProvider;
use storage::local::{LocalStorage, NewLocalStorageError};

pub(super) mod storage;
pub(super) struct Provider {
    crypto: RustCrypto,
    storage: LocalStorage,
}

impl Provider {
    pub(super) fn new() -> Result<Self, NewLocalStorageError> {
        Ok(Self {
            crypto: RustCrypto::default(),
            storage: LocalStorage::new()?,
        })
    }
}

impl OpenMlsProvider for Provider {
    type CryptoProvider = RustCrypto;

    type RandProvider = RustCrypto;

    type StorageProvider = LocalStorage;

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
