use std::collections::HashSet;

use base64::{Engine, prelude::BASE64_URL_SAFE_NO_PAD};
use wasm_bindgen::JsValue;

mod key {

    pub(super) fn group_join_configuration(group_id: &str) -> String {
        format!("openmls/groups/{group_id}/join-configuration")
    }

    pub(super) fn group_leaf_nodes(group_id: &str) -> String {
        format!("openmls/groups/{group_id}/leaf-nodes")
    }

    pub(super) fn group_proposal_references(group_id: &str) -> String {
        format!("openmls/groups/{group_id}/proposals/references")
    }

    pub(super) fn group_proposal(group_id: &str, proposal_reference: &str) -> String {
        format!("openmls/groups/{group_id}/proposals/{proposal_reference}")
    }

    pub(super) fn group_tree(group_id: &str) -> String {
        format!("openmls/groups/{group_id}/tree")
    }

    pub(super) fn group_interim_transcript_hash(group_id: &str) -> String {
        format!("openmls/groups/{group_id}/interim-transcription-hash")
    }

    pub(super) fn group_context(group_id: &str) -> String {
        format!("openmls/groups/{group_id}/context")
    }

    pub(super) fn group_confirmation_tag(group_id: &str) -> String {
        format!("openmls/groups/{group_id}/confirmation-tag")
    }

    pub(super) fn group_state(group_id: &str) -> String {
        format!("openmls/groups/{group_id}/state")
    }

    pub(super) fn group_message_secrets(group_id: &str) -> String {
        format!("openmls/groups/{group_id}/message-secrets")
    }

    pub(super) fn group_resumption_pre_shared_key_store(group_id: &str) -> String {
        format!("openmls/groups/{group_id}/pre-shared-key-store")
    }

    pub(super) fn group_leaf_index(group_id: &str) -> String {
        format!("openmls/groups/{group_id}/leaf-index")
    }

    pub(super) fn group_epoch_secrets(group_id: &str) -> String {
        format!("openmls/groups/{group_id}/epoch-secrets")
    }

    pub(super) fn signature_key_pair(public_key: &str) -> String {
        format!("openmls/signature-keys/{public_key}")
    }

    pub(super) fn encryption_key_pair(public_key: &str) -> String {
        format!("openmls/encryption-keys/{public_key}")
    }

    pub(super) fn group_epochs(group_id: &str) -> String {
        format!("openmls/groups/{group_id}/epochs")
    }

    pub(super) fn group_epoch_leaf_indices(group_id: &str, epoch: &str) -> String {
        format!("openmls/groups/{group_id}/epochs/{epoch}/leafs/indices")
    }

    pub(super) fn group_epoch_leaf_key_pairs(
        group_id: &str,
        epoch: &str,
        leaf_index: u32,
    ) -> String {
        format!("openmls/groups/{group_id}/epochs/{epoch}/leafs/{leaf_index}/key-pairs")
    }

    pub(super) fn key_package(hash_reference: &str) -> String {
        format!("openmls/key-packages/{hash_reference}")
    }

    pub(super) fn pre_shared_key(pre_shared_key_id: &str) -> String {
        format!("openmls/pre-shared-keys/{pre_shared_key_id}")
    }
}
/// A local storage OpenMLS storage provider that should probably not be used.
/// But it is the only persistent synchronous storage in the browser.
pub(crate) struct LocalStorage(web_sys::Storage);

#[derive(Debug, thiserror::Error)]
pub(crate) enum NewLocalStorageError {
    #[error("Could not get the global window object")]
    NoWindow,
    #[error("Error getting local storage")]
    LocalStorageError(JsValue),
    #[error("No local storage object on window")]
    NoLocalStorageObject,
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum GetItemError {
    #[error("Error getting item")]
    GetItemError(JsValue),
    #[error("Error deserializing item")]
    DeserializationError(#[from] DestringalizeError),
}

#[derive(Debug, thiserror::Error)]
#[error("Error removing item")]
pub(crate) struct RemoveItemError(JsValue);

impl LocalStorage {
    pub(crate) fn new() -> Result<Self, NewLocalStorageError> {
        web_sys::window()
            .ok_or(NewLocalStorageError::NoWindow)?
            .local_storage()
            .map_err(NewLocalStorageError::LocalStorageError)?
            .ok_or(NewLocalStorageError::NoLocalStorageObject)
            .map(Self)
    }

    fn get_item<T: serde::de::DeserializeOwned>(
        &self,
        key: &str,
    ) -> Result<Option<T>, GetItemError> {
        let encoded = self.0.get_item(&key).map_err(GetItemError::GetItemError)?;

        let result = encoded.map(|item| destringalize::<T>(&item)).transpose()?;
        Ok(result)
    }

    fn remove_item(&self, key: &str) -> Result<(), RemoveItemError> {
        self.0.remove_item(&key).map_err(RemoveItemError)
    }

    fn set_item(&self, key: &str, value: &impl Stringalize) -> Result<(), SetItemError> {
        let value = value.stringalize()?;

        self.0
            .set_item(&key, &value)
            .map_err(SetItemError::SetItemError)
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum StringalizeError {
    #[error("Error serializing with postcard: {0}")]
    PostcardError(#[from] postcard::Error),
}

/// Silly implementation, silly name
trait Stringalize {
    fn stringalize(&self) -> Result<String, StringalizeError>;
}

impl<K: serde::Serialize> Stringalize for K {
    fn stringalize(&self) -> Result<String, StringalizeError> {
        let serialized = postcard::to_allocvec(&self)?;
        Ok(BASE64_URL_SAFE_NO_PAD.encode(&serialized))
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum DestringalizeError {
    #[error("Error decoding base64")]
    Base64Error(#[from] base64::DecodeError),
    #[error("Error deserializing with postcard: {0}")]
    PostcardError(#[from] postcard::Error),
}

fn destringalize<K: serde::de::DeserializeOwned>(encoded: &str) -> Result<K, DestringalizeError> {
    let decoded = BASE64_URL_SAFE_NO_PAD.decode(encoded)?;
    let value = postcard::from_bytes(&decoded)?;
    Ok(value)
}

const VERSION: u16 = 1;

#[derive(Debug, thiserror::Error)]
pub(crate) enum SetItemError {
    #[error("Error stringalizing: {0}")]
    StringalizeError(#[from] StringalizeError),
    #[error("Error setting item")]
    SetItemError(JsValue),
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum LocalStorageError {
    #[error("Error stringalizing: {0}")]
    StringalizeError(#[from] StringalizeError),
    #[error("Error destringalizing: {0}")]
    DestringalizeError(#[from] DestringalizeError),
    #[error("Error setting item: {0}")]
    SetItemError(#[from] SetItemError),
    #[error("Error getting item")]
    GetItemError(#[from] GetItemError),
    #[error("Error removing item: {0}")]
    RemoveItemError(#[from] RemoveItemError),
    #[error("Found no proposal for proposal reference: {proposal_reference}")]
    ProposalNotFoundError { proposal_reference: String },
}

impl openmls_traits::storage::StorageProvider<VERSION> for LocalStorage {
    type Error = LocalStorageError;

    fn write_mls_join_config<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        MlsGroupJoinConfig: openmls_traits::storage::traits::MlsGroupJoinConfig<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        configuration: &MlsGroupJoinConfig,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_join_configuration = key::group_join_configuration(&group_id);
        self.set_item(&group_join_configuration, configuration)?;
        Ok(())
    }

    fn append_own_leaf_node<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        LeafNode: openmls_traits::storage::traits::LeafNode<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        leaf_node: &LeafNode,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_leaf_nodes = key::group_leaf_nodes(&group_id);

        let mut leaf_nodes: Vec<String> = self.get_item(&group_leaf_nodes)?.unwrap_or_default();
        // Leaf node is not clonable so we need to convert it to a string which means double encoding
        let leaf_node = leaf_node.stringalize()?;
        leaf_nodes.push(leaf_node);
        self.set_item(&group_leaf_nodes, &leaf_nodes)?;

        Ok(())
    }

    fn queue_proposal<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ProposalRef: openmls_traits::storage::traits::ProposalRef<VERSION>,
        QueuedProposal: openmls_traits::storage::traits::QueuedProposal<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        proposal_reference: &ProposalRef,
        proposal: &QueuedProposal,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_proposal_references = key::group_proposal_references(&group_id);
        let mut references: HashSet<String> = self
            .get_item(&group_proposal_references)?
            .unwrap_or_default();

        let proposal_reference = proposal_reference.stringalize()?;
        references.insert(proposal_reference.clone());

        self.set_item(&group_proposal_references, &references)?;

        let group_proposal = key::group_proposal(&group_id, &proposal_reference);
        self.set_item(&group_proposal, proposal)?;
        Ok(())
    }

    fn write_tree<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        TreeSync: openmls_traits::storage::traits::TreeSync<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        tree: &TreeSync,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_tree = key::group_tree(&group_id);
        self.set_item(&group_tree, tree)?;
        Ok(())
    }

    fn write_interim_transcript_hash<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        InterimTranscriptHash: openmls_traits::storage::traits::InterimTranscriptHash<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        interim_transcript_hash: &InterimTranscriptHash,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_interim_transcript_hash = key::group_interim_transcript_hash(&group_id);
        self.set_item(&group_interim_transcript_hash, interim_transcript_hash)?;
        Ok(())
    }

    fn write_context<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        GroupContext: openmls_traits::storage::traits::GroupContext<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        group_context: &GroupContext,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_context_key = key::group_context(&group_id);
        self.set_item(&group_context_key, group_context)?;
        Ok(())
    }

    fn write_confirmation_tag<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ConfirmationTag: openmls_traits::storage::traits::ConfirmationTag<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        confirmation_tag: &ConfirmationTag,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_confirmation_tag_key = key::group_confirmation_tag(&group_id);
        self.set_item(&group_confirmation_tag_key, confirmation_tag)?;

        Ok(())
    }

    fn write_group_state<
        GroupState: openmls_traits::storage::traits::GroupState<VERSION>,
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        group_state: &GroupState,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_state_key = key::group_state(&group_id);
        self.set_item(&group_state_key, group_state)?;

        Ok(())
    }

    fn write_message_secrets<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        MessageSecrets: openmls_traits::storage::traits::MessageSecrets<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        message_secrets: &MessageSecrets,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let message_secrets_key = key::group_message_secrets(&group_id);
        self.set_item(&message_secrets_key, message_secrets)?;

        Ok(())
    }

    fn write_resumption_psk_store<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ResumptionPskStore: openmls_traits::storage::traits::ResumptionPskStore<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        resumption_pre_shared_key_store: &ResumptionPskStore,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_resumption_pre_shared_key_store =
            key::group_resumption_pre_shared_key_store(&group_id);
        self.set_item(
            &group_resumption_pre_shared_key_store,
            resumption_pre_shared_key_store,
        )?;

        Ok(())
    }

    fn write_own_leaf_index<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        LeafNodeIndex: openmls_traits::storage::traits::LeafNodeIndex<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        own_leaf_index: &LeafNodeIndex,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_own_leaf_index = key::group_leaf_index(&group_id);
        self.set_item(&group_own_leaf_index, own_leaf_index)?;

        Ok(())
    }

    fn write_group_epoch_secrets<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        GroupEpochSecrets: openmls_traits::storage::traits::GroupEpochSecrets<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        group_epoch_secrets: &GroupEpochSecrets,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_epoch_secrets_key = key::group_epoch_secrets(&group_id);
        self.set_item(&group_epoch_secrets_key, group_epoch_secrets)?;

        Ok(())
    }

    fn write_signature_key_pair<
        SignaturePublicKey: openmls_traits::storage::traits::SignaturePublicKey<VERSION>,
        SignatureKeyPair: openmls_traits::storage::traits::SignatureKeyPair<VERSION>,
    >(
        &self,
        public_key: &SignaturePublicKey,
        signature_key_pair: &SignatureKeyPair,
    ) -> Result<(), Self::Error> {
        let public_key = public_key.stringalize()?;
        let signature_key_pairs = key::signature_key_pair(&public_key);
        self.set_item(&signature_key_pairs, signature_key_pair)?;

        Ok(())
    }

    fn write_encryption_key_pair<
        EncryptionKey: openmls_traits::storage::traits::EncryptionKey<VERSION>,
        HpkeKeyPair: openmls_traits::storage::traits::HpkeKeyPair<VERSION>,
    >(
        &self,
        public_key: &EncryptionKey,
        key_pair: &HpkeKeyPair,
    ) -> Result<(), Self::Error> {
        let public_key = public_key.stringalize()?;
        let encryption_key_pairs = key::encryption_key_pair(&public_key);
        self.set_item(&encryption_key_pairs, key_pair)?;

        Ok(())
    }

    fn write_encryption_epoch_key_pairs<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        EpochKey: openmls_traits::storage::traits::EpochKey<VERSION>,
        HpkeKeyPair: openmls_traits::storage::traits::HpkeKeyPair<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        epoch: &EpochKey,
        leaf_index: u32,
        key_pairs: &[HpkeKeyPair],
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_epochs = key::group_epochs(&group_id);

        let mut epochs: HashSet<String> = self.get_item(&group_epochs)?.unwrap_or_default();
        let epoch = epoch.stringalize()?;
        epochs.insert(epoch.clone());
        self.set_item(&group_epochs, &epochs)?;

        let group_epoch_leaf_indices = key::group_epoch_leaf_indices(&group_id, &epoch);
        let mut leaf_indices: HashSet<u32> = self
            .get_item(&group_epoch_leaf_indices)?
            .unwrap_or_default();

        leaf_indices.insert(leaf_index);
        self.set_item(&group_epoch_leaf_indices, &leaf_indices)?;

        let group_epoch_leaf_key_pairs =
            key::group_epoch_leaf_key_pairs(&group_id, &epoch, leaf_index);
        self.set_item(&group_epoch_leaf_key_pairs, &key_pairs)?;

        Ok(())
    }

    fn write_key_package<
        HashReference: openmls_traits::storage::traits::HashReference<VERSION>,
        KeyPackage: openmls_traits::storage::traits::KeyPackage<VERSION>,
    >(
        &self,
        hash_reference: &HashReference,
        key_package: &KeyPackage,
    ) -> Result<(), Self::Error> {
        let hash_reference = hash_reference.stringalize()?;
        let key_package_key = key::key_package(&hash_reference);
        self.set_item(&key_package_key, key_package)?;
        Ok(())
    }

    fn write_psk<
        PskId: openmls_traits::storage::traits::PskId<VERSION>,
        PskBundle: openmls_traits::storage::traits::PskBundle<VERSION>,
    >(
        &self,
        pre_shared_key_id: &PskId,
        pre_shared_key: &PskBundle,
    ) -> Result<(), Self::Error> {
        let pre_shared_key_id = pre_shared_key_id.stringalize()?;
        let pre_shared_key_key = key::pre_shared_key(&pre_shared_key_id);
        self.set_item(&pre_shared_key_key, pre_shared_key)?;
        Ok(())
    }

    fn mls_group_join_config<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        MlsGroupJoinConfig: openmls_traits::storage::traits::MlsGroupJoinConfig<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<MlsGroupJoinConfig>, Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_join_configuration = key::group_join_configuration(&group_id);
        let configuration = self.get_item(&group_join_configuration)?;

        Ok(configuration)
    }

    fn own_leaf_nodes<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        LeafNode: openmls_traits::storage::traits::LeafNode<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Vec<LeafNode>, Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_own_leaf_nodes = key::group_leaf_nodes(&group_id);
        let leaf_nodes: Option<Vec<String>> = self.get_item(&group_own_leaf_nodes)?;

        let Some(leaf_nodes) = leaf_nodes else {
            return Ok(Vec::new());
        };

        let mut refined_leaf_nodes = Vec::with_capacity(leaf_nodes.len());

        for leaf_node in leaf_nodes {
            let refined_leaf_node = destringalize(&leaf_node)?;
            refined_leaf_nodes.push(refined_leaf_node);
        }

        Ok(refined_leaf_nodes)
    }

    fn queued_proposal_refs<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ProposalRef: openmls_traits::storage::traits::ProposalRef<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Vec<ProposalRef>, Self::Error> {
        let group_id = group_id.stringalize()?;
        let path = key::group_proposal_references(&group_id);
        let proposal_references: Option<Vec<String>> = self.get_item(&path)?;

        let Some(proposal_refs) = proposal_references else {
            return Ok(Vec::new());
        };

        let mut refined_proposal_refs = Vec::with_capacity(proposal_refs.len());

        for proposal_ref in proposal_refs {
            let refined_proposal_ref = destringalize(&proposal_ref)?;
            refined_proposal_refs.push(refined_proposal_ref);
        }

        Ok(refined_proposal_refs)
    }

    fn queued_proposals<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ProposalRef: openmls_traits::storage::traits::ProposalRef<VERSION>,
        QueuedProposal: openmls_traits::storage::traits::QueuedProposal<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Vec<(ProposalRef, QueuedProposal)>, Self::Error> {
        let proposal_references = self.queued_proposal_refs::<GroupId, ProposalRef>(group_id)?;

        let group_id = group_id.stringalize()?;
        let mut proposals = Vec::with_capacity(proposal_references.len());

        for proposal_reference in proposal_references {
            let proposal_reference_key = proposal_reference.stringalize()?;
            let group_proposal = key::group_proposal(&group_id, &proposal_reference_key);
            let proposal: QueuedProposal = self.get_item(&group_proposal)?.ok_or(
                LocalStorageError::ProposalNotFoundError {
                    proposal_reference: proposal_reference_key,
                },
            )?;

            proposals.push((proposal_reference, proposal));
        }

        Ok(proposals)
    }

    fn tree<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        TreeSync: openmls_traits::storage::traits::TreeSync<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<TreeSync>, Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_tree = key::group_tree(&group_id);
        let tree = self.get_item(&group_tree)?;

        Ok(tree)
    }

    fn group_context<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        GroupContext: openmls_traits::storage::traits::GroupContext<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<GroupContext>, Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_context = key::group_context(&group_id);
        let context = self.get_item(&group_context)?;

        Ok(context)
    }

    fn interim_transcript_hash<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        InterimTranscriptHash: openmls_traits::storage::traits::InterimTranscriptHash<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<InterimTranscriptHash>, Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_interim_transcript_hash = key::group_interim_transcript_hash(&group_id);
        let hash = self.get_item(&group_interim_transcript_hash)?;

        Ok(hash)
    }

    fn confirmation_tag<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ConfirmationTag: openmls_traits::storage::traits::ConfirmationTag<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<ConfirmationTag>, Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_confirmation_tag = key::group_confirmation_tag(&group_id);
        let tag = self.get_item(&group_confirmation_tag)?;

        Ok(tag)
    }

    fn group_state<
        GroupState: openmls_traits::storage::traits::GroupState<VERSION>,
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<GroupState>, Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_state = key::group_state(&group_id);
        let state = self.get_item(&group_state)?;

        Ok(state)
    }

    fn message_secrets<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        MessageSecrets: openmls_traits::storage::traits::MessageSecrets<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<MessageSecrets>, Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_message_secrets = key::group_message_secrets(&group_id);
        let secrets = self.get_item(&group_message_secrets)?;

        Ok(secrets)
    }

    fn resumption_psk_store<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ResumptionPskStore: openmls_traits::storage::traits::ResumptionPskStore<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<ResumptionPskStore>, Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_resumption_pre_shared_key_store =
            key::group_resumption_pre_shared_key_store(&group_id);
        let store = self.get_item(&group_resumption_pre_shared_key_store)?;

        Ok(store)
    }

    fn own_leaf_index<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        LeafNodeIndex: openmls_traits::storage::traits::LeafNodeIndex<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<LeafNodeIndex>, Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_leaf_index = key::group_leaf_index(&group_id);
        let index = self.get_item(&group_leaf_index)?;

        Ok(index)
    }

    fn group_epoch_secrets<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        GroupEpochSecrets: openmls_traits::storage::traits::GroupEpochSecrets<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<GroupEpochSecrets>, Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_epoch_secrets = key::group_epoch_secrets(&group_id);
        let secrets = self.get_item(&group_epoch_secrets)?;

        Ok(secrets)
    }

    fn signature_key_pair<
        SignaturePublicKey: openmls_traits::storage::traits::SignaturePublicKey<VERSION>,
        SignatureKeyPair: openmls_traits::storage::traits::SignatureKeyPair<VERSION>,
    >(
        &self,
        public_key: &SignaturePublicKey,
    ) -> Result<Option<SignatureKeyPair>, Self::Error> {
        let public_key = public_key.stringalize()?;
        let signature_key_pair = key::signature_key_pair(&public_key);
        let key_pair = self.get_item(&signature_key_pair)?;

        Ok(key_pair)
    }

    fn encryption_key_pair<
        HpkeKeyPair: openmls_traits::storage::traits::HpkeKeyPair<VERSION>,
        EncryptionKey: openmls_traits::storage::traits::EncryptionKey<VERSION>,
    >(
        &self,
        public_key: &EncryptionKey,
    ) -> Result<Option<HpkeKeyPair>, Self::Error> {
        let public_key = public_key.stringalize()?;
        let encryption_key_pair = key::encryption_key_pair(&public_key);
        let key_pair = self.get_item(&encryption_key_pair)?;

        Ok(key_pair)
    }

    fn encryption_epoch_key_pairs<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        EpochKey: openmls_traits::storage::traits::EpochKey<VERSION>,
        HpkeKeyPair: openmls_traits::storage::traits::HpkeKeyPair<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        epoch: &EpochKey,
        leaf_index: u32,
    ) -> Result<Vec<HpkeKeyPair>, Self::Error> {
        let group_id = group_id.stringalize()?;
        let epoch = epoch.stringalize()?;
        let group_epoch_leaf_key_pairs =
            key::group_epoch_leaf_key_pairs(&group_id, &epoch, leaf_index);
        let key_pairs = self
            .get_item(&group_epoch_leaf_key_pairs)?
            .unwrap_or_default();

        Ok(key_pairs)
    }

    fn key_package<
        KeyPackageRef: openmls_traits::storage::traits::HashReference<VERSION>,
        KeyPackage: openmls_traits::storage::traits::KeyPackage<VERSION>,
    >(
        &self,
        hash_reference: &KeyPackageRef,
    ) -> Result<Option<KeyPackage>, Self::Error> {
        let hash_reference = hash_reference.stringalize()?;
        let key_package = key::key_package(&hash_reference);
        let key_package = self.get_item(&key_package)?;

        Ok(key_package)
    }

    fn psk<
        PskBundle: openmls_traits::storage::traits::PskBundle<VERSION>,
        PskId: openmls_traits::storage::traits::PskId<VERSION>,
    >(
        &self,
        pre_shared_key_id: &PskId,
    ) -> Result<Option<PskBundle>, Self::Error> {
        let pre_shared_key_id = pre_shared_key_id.stringalize()?;
        let pre_shared_key = key::pre_shared_key(&pre_shared_key_id);
        let pre_shared_key_bundle = self.get_item(&pre_shared_key)?;

        Ok(pre_shared_key_bundle)
    }

    fn remove_proposal<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ProposalRef: openmls_traits::storage::traits::ProposalRef<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        proposal_reference: &ProposalRef,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_proposal_references = key::group_proposal_references(&group_id);
        let proposal_reference = proposal_reference.stringalize()?;

        let references: Option<HashSet<String>> = self.get_item(&group_proposal_references)?;

        if let Some(mut references) = references {
            references.remove(&proposal_reference);
            self.set_item(&group_proposal_references, &references)?;
        }

        let group_proposal = key::group_proposal(&group_id, &proposal_reference);
        self.remove_item(&group_proposal)?;

        Ok(())
    }

    fn delete_own_leaf_nodes<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_leaf_nodes = key::group_leaf_nodes(&group_id);

        self.remove_item(&group_leaf_nodes)?;

        Ok(())
    }

    fn delete_group_config<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_join_configuration = key::group_join_configuration(&group_id);
        self.remove_item(&group_join_configuration)?;

        Ok(())
    }

    fn delete_tree<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_tree = key::group_tree(&group_id);

        self.remove_item(&group_tree)?;

        Ok(())
    }

    fn delete_confirmation_tag<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_confirmation_tag = key::group_confirmation_tag(&group_id);
        self.remove_item(&group_confirmation_tag)?;

        Ok(())
    }

    fn delete_group_state<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_state = key::group_state(&group_id);
        self.remove_item(&group_state)?;

        Ok(())
    }

    fn delete_context<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_context = key::group_context(&group_id);
        self.remove_item(&group_context)?;

        Ok(())
    }

    fn delete_interim_transcript_hash<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_interim_transcript_hash = key::group_interim_transcript_hash(&group_id);
        self.remove_item(&group_interim_transcript_hash)?;

        Ok(())
    }

    fn delete_message_secrets<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_message_secrets = key::group_message_secrets(&group_id);
        self.remove_item(&group_message_secrets)?;
        Ok(())
    }

    fn delete_all_resumption_psk_secrets<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_resumption_pre_shared_key_store =
            key::group_resumption_pre_shared_key_store(&group_id);
        self.remove_item(&group_resumption_pre_shared_key_store)?;

        Ok(())
    }

    fn delete_own_leaf_index<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_leaf_index = key::group_leaf_index(&group_id);
        self.remove_item(&group_leaf_index)?;
        Ok(())
    }

    fn delete_group_epoch_secrets<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let group_epoch_secrets = key::group_epoch_secrets(&group_id);
        self.remove_item(&group_epoch_secrets)?;
        Ok(())
    }

    fn clear_proposal_queue<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ProposalRef: openmls_traits::storage::traits::ProposalRef<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let proposal_references = self.queued_proposal_refs::<GroupId, ProposalRef>(group_id)?;
        let group_id = group_id.stringalize()?;
        let group_proposal_references = key::group_proposal_references(&group_id);

        for proposal_reference in proposal_references {
            let proposal_reference = proposal_reference.stringalize()?;
            let group_proposal = key::group_proposal(&group_id, &proposal_reference);
            self.remove_item(&group_proposal)?;
        }

        self.remove_item(&group_proposal_references)?;

        Ok(())
    }

    fn delete_signature_key_pair<
        SignaturePublicKey: openmls_traits::storage::traits::SignaturePublicKey<VERSION>,
    >(
        &self,
        public_key: &SignaturePublicKey,
    ) -> Result<(), Self::Error> {
        let public_key = public_key.stringalize()?;
        let signature_key_pair = key::signature_key_pair(&public_key);
        self.remove_item(&signature_key_pair)?;
        Ok(())
    }

    fn delete_encryption_key_pair<
        EncryptionKey: openmls_traits::storage::traits::EncryptionKey<VERSION>,
    >(
        &self,
        public_key: &EncryptionKey,
    ) -> Result<(), Self::Error> {
        let public_key = public_key.stringalize()?;
        let encryption_key_pair = key::encryption_key_pair(&public_key);
        self.remove_item(&encryption_key_pair)?;
        Ok(())
    }

    fn delete_encryption_epoch_key_pairs<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        EpochKey: openmls_traits::storage::traits::EpochKey<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        epoch: &EpochKey,
        leaf_index: u32,
    ) -> Result<(), Self::Error> {
        let group_id = group_id.stringalize()?;
        let epoch = epoch.stringalize()?;
        let group_epoch_leaf_key_pairs =
            key::group_epoch_leaf_key_pairs(&group_id, &epoch, leaf_index);

        self.remove_item(&group_epoch_leaf_key_pairs)?;

        let group_epoch_leaf_indices = key::group_epoch_leaf_indices(&group_id, &epoch);
        let leaf_indices: Option<HashSet<u32>> = self.get_item(&group_epoch_leaf_indices)?;

        if let Some(mut leaf_indices) = leaf_indices {
            leaf_indices.remove(&leaf_index);

            if leaf_indices.is_empty() {
                self.remove_item(&group_epoch_leaf_indices)?;
            } else {
                self.set_item(&group_epoch_leaf_indices, &leaf_indices)?;
            }
        }

        let group_epochs = key::group_epochs(&group_id);
        let epochs: Option<HashSet<String>> = self.get_item(&group_epochs)?;

        if let Some(mut epochs) = epochs {
            epochs.remove(&epoch);

            if epochs.is_empty() {
                self.remove_item(&group_epochs)?;
            } else {
                self.set_item(&group_epochs, &epochs)?;
            }
        }

        Ok(())
    }

    fn delete_key_package<
        KeyPackageRef: openmls_traits::storage::traits::HashReference<VERSION>,
    >(
        &self,
        hash_reference: &KeyPackageRef,
    ) -> Result<(), Self::Error> {
        let hash_reference = hash_reference.stringalize()?;
        let key_package = key::key_package(&hash_reference);
        self.remove_item(&key_package)?;

        Ok(())
    }

    fn delete_psk<PskKey: openmls_traits::storage::traits::PskId<VERSION>>(
        &self,
        pre_shared_key_id: &PskKey,
    ) -> Result<(), Self::Error> {
        let pre_shared_key_id = pre_shared_key_id.stringalize()?;
        let pre_shared_key = key::pre_shared_key(&pre_shared_key_id);
        self.remove_item(&pre_shared_key)?;

        Ok(())
    }
}
