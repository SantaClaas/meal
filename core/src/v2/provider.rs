use std::collections::HashMap;

use openmls_rust_crypto::{MemoryStorage, RustCrypto};
use openmls_traits::OpenMlsProvider;
use serde::{Deserialize, Serialize};

/// New type wrapper to allow serialization and deserialization
#[derive(Default)]
struct Storage(MemoryStorage);

impl<'de> Deserialize<'de> for Storage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // deserializer.deserialize_map(visitor)
        let map: HashMap<Vec<u8>, Vec<u8>> = Deserialize::deserialize(deserializer)?;
        let storage = MemoryStorage::default();
        {
            let mut lock = storage
                .values
                .write()
                .expect("Storage is freshly create and should not have been poisoned");

            *lock = map;
        }

        Ok(Storage(storage))
    }
}

impl Serialize for Storage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let map = self
            .0
            .values
            .read()
            .map_err(|_| serde::ser::Error::custom("Lock poisoned"))?;

        map.serialize(serializer)
    }
}

#[derive(Deserialize, Serialize)]
pub(super) struct Provider {
    #[serde(default, skip)]
    crypto: RustCrypto,
    storage: Storage,
}

impl Provider {
    pub(super) fn new() -> Self {
        Self {
            crypto: RustCrypto::default(),
            storage: Storage::default(),
        }
    }
}

impl OpenMlsProvider for Provider {
    type CryptoProvider = RustCrypto;

    type RandProvider = RustCrypto;

    type StorageProvider = MemoryStorage;

    fn storage(&self) -> &Self::StorageProvider {
        &self.storage.0
    }

    fn crypto(&self) -> &Self::CryptoProvider {
        &self.crypto
    }

    fn rand(&self) -> &Self::RandProvider {
        &self.crypto
    }
}
