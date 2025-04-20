mod operation;

use operation::Operation;
use wasm_bindgen::JsValue;
use web_sys::js_sys;

const VERSION: u16 = 1;

#[derive(Debug, thiserror::Error)]
pub(crate) enum InstructionError {
    #[error("Error serializing value to JsValue: {0}")]
    Serialize(#[from] serde_wasm_bindgen::Error),
    #[error("Error calling JavaScript function")]
    JsError(JsValue),
    #[error("Error deserializing response value from JsValue: {0}")]
    Deserialize(serde_wasm_bindgen::Error),
}

pub(crate) struct Provider {
    /// The function in JavaScript land to execute the store operations
    bridge: js_sys::Function,
}

impl Provider {
    pub(super) fn new(bridge: js_sys::Function) -> Self {
        Self { bridge }
    }
}

impl openmls_traits::storage::StorageProvider<VERSION> for Provider {
    type Error = InstructionError;

    fn write_mls_join_config<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        MlsGroupJoinConfig: openmls_traits::storage::traits::MlsGroupJoinConfig<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        configuration: &MlsGroupJoinConfig,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::WriteMlsJoinConfig)?;
        let group_id = serde_wasm_bindgen::to_value(&group_id)?;
        let configuration = serde_wasm_bindgen::to_value(&configuration)?;

        self.bridge
            .call3(&JsValue::NULL, &instruction, &group_id, &configuration)
            .map_err(InstructionError::JsError)?;
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
        let instruction = serde_wasm_bindgen::to_value(&Operation::AppendOwnLeafNode)?;
        let group_id = serde_wasm_bindgen::to_value(&group_id)?;
        let leaf_node = serde_wasm_bindgen::to_value(&leaf_node)?;

        self.bridge
            .call3(&JsValue::NULL, &instruction, &group_id, &leaf_node)
            .map_err(InstructionError::JsError)?;
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
        let instruction = serde_wasm_bindgen::to_value(&Operation::QueueProposal)?;
        let group_id = serde_wasm_bindgen::to_value(&group_id)?;
        let proposal_reference = serde_wasm_bindgen::to_value(&proposal_reference)?;
        let keys = js_sys::Array::of2(&group_id, &proposal_reference);
        let proposal = serde_wasm_bindgen::to_value(&proposal)?;

        self.bridge
            .call3(&JsValue::NULL, &instruction, &keys, &proposal)
            .map_err(InstructionError::JsError)?;
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
        let instruction = serde_wasm_bindgen::to_value(&Operation::WriteTree)?;
        let group_id = serde_wasm_bindgen::to_value(&group_id)?;
        let tree = serde_wasm_bindgen::to_value(&tree)?;

        self.bridge
            .call3(&JsValue::NULL, &instruction, &group_id, &tree)
            .map_err(InstructionError::JsError)?;

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
        let instruction = serde_wasm_bindgen::to_value(&Operation::WriteInterimTranscriptHash)?;
        let group_id = serde_wasm_bindgen::to_value(&group_id)?;
        let interim_transcript_hash = serde_wasm_bindgen::to_value(&interim_transcript_hash)?;

        self.bridge
            .call3(
                &JsValue::NULL,
                &instruction,
                &group_id,
                &interim_transcript_hash,
            )
            .map_err(InstructionError::JsError)?;

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
        let instruction = serde_wasm_bindgen::to_value(&Operation::WriteGroupContext)?;
        let group_id = serde_wasm_bindgen::to_value(&group_id)?;
        let group_context = serde_wasm_bindgen::to_value(&group_context)?;

        self.bridge
            .call3(&JsValue::NULL, &instruction, &group_id, &group_context)
            .map_err(InstructionError::JsError)?;

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
        let instruction = serde_wasm_bindgen::to_value(&Operation::WriteConfirmationTag)?;
        let group_id = serde_wasm_bindgen::to_value(&group_id)?;
        let confirmation_tag = serde_wasm_bindgen::to_value(&confirmation_tag)?;

        self.bridge
            .call3(&JsValue::NULL, &instruction, &group_id, &confirmation_tag)
            .map_err(InstructionError::JsError)?;

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
        let instruction = serde_wasm_bindgen::to_value(&Operation::WriteGroupState)?;
        let group_id = serde_wasm_bindgen::to_value(&group_id)?;
        let group_state = serde_wasm_bindgen::to_value(&group_state)?;

        self.bridge
            .call3(&JsValue::NULL, &instruction, &group_id, &group_state)
            .map_err(InstructionError::JsError)?;

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
        let instruction = serde_wasm_bindgen::to_value(&Operation::WriteMessageSecrets)?;
        let group_id = serde_wasm_bindgen::to_value(&group_id)?;
        let message_secrets = serde_wasm_bindgen::to_value(&message_secrets)?;

        self.bridge
            .call3(&JsValue::NULL, &instruction, &group_id, &message_secrets)
            .map_err(InstructionError::JsError)?;

        Ok(())
    }

    fn write_resumption_psk_store<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ResumptionPskStore: openmls_traits::storage::traits::ResumptionPskStore<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        resumption_psk_store: &ResumptionPskStore,
    ) -> Result<(), Self::Error> {
        let instruction =
            serde_wasm_bindgen::to_value(&Operation::WriteResumptionPreSharedKeyStore)?;
        let group_id = serde_wasm_bindgen::to_value(&group_id)?;
        let resumption_psk_store = serde_wasm_bindgen::to_value(&resumption_psk_store)?;

        self.bridge
            .call3(
                &JsValue::NULL,
                &instruction,
                &group_id,
                &resumption_psk_store,
            )
            .map_err(InstructionError::JsError)?;

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
        let instruction = serde_wasm_bindgen::to_value(&Operation::WriteOwnLeafIndex)?;
        let group_id = serde_wasm_bindgen::to_value(&group_id)?;
        let own_leaf_index = serde_wasm_bindgen::to_value(&own_leaf_index)?;

        self.bridge
            .call3(&JsValue::NULL, &instruction, &group_id, &own_leaf_index)
            .map_err(InstructionError::JsError)?;

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
        let instruction = serde_wasm_bindgen::to_value(&Operation::WriteGroupEpochSecrets)?;
        let group_id = serde_wasm_bindgen::to_value(&group_id)?;
        let group_epoch_secrets = serde_wasm_bindgen::to_value(&group_epoch_secrets)?;

        self.bridge
            .call3(
                &JsValue::NULL,
                &instruction,
                &group_id,
                &group_epoch_secrets,
            )
            .map_err(InstructionError::JsError)?;

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
        let instruction = serde_wasm_bindgen::to_value(&Operation::WriteSignatureKeyPair)?;
        let public_key = serde_wasm_bindgen::to_value(public_key)?;
        let signature_key_pair = serde_wasm_bindgen::to_value(signature_key_pair)?;

        self.bridge
            .call3(
                &JsValue::NULL,
                &instruction,
                &public_key,
                &signature_key_pair,
            )
            .map_err(InstructionError::JsError)?;

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
        let instruction = serde_wasm_bindgen::to_value(&Operation::WriteEncryptionKeyPair)?;
        let public_key = serde_wasm_bindgen::to_value(public_key)?;
        let key_pair = serde_wasm_bindgen::to_value(key_pair)?;

        self.bridge
            .call3(&JsValue::NULL, &instruction, &public_key, &key_pair)
            .map_err(InstructionError::JsError)?;

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
        let instruction = serde_wasm_bindgen::to_value(&Operation::WriteEncryptionEpochKeyPairs)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;
        let epoch = serde_wasm_bindgen::to_value(epoch)?;
        let leaf_index = serde_wasm_bindgen::to_value(&leaf_index)?;
        let keys = js_sys::Array::of3(&group_id, &epoch, &leaf_index);

        let key_pairs = serde_wasm_bindgen::to_value(key_pairs)?;

        self.bridge
            .call3(&JsValue::NULL, &instruction, &keys, &key_pairs)
            .map_err(InstructionError::JsError)?;

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
        let instruction = serde_wasm_bindgen::to_value(&Operation::WriteKeyPackage)?;

        let hash_reference = serde_wasm_bindgen::to_value(hash_reference)?;
        let key_package = serde_wasm_bindgen::to_value(key_package)?;

        self.bridge
            .call3(&JsValue::NULL, &instruction, &hash_reference, &key_package)
            .map_err(InstructionError::JsError)?;

        Ok(())
    }

    fn write_psk<
        PskId: openmls_traits::storage::traits::PskId<VERSION>,
        PskBundle: openmls_traits::storage::traits::PskBundle<VERSION>,
    >(
        &self,
        psk_id: &PskId,
        pre_shared_key: &PskBundle,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::WritePreSharedKey)?;

        let psk_id = serde_wasm_bindgen::to_value(psk_id)?;
        let psk = serde_wasm_bindgen::to_value(pre_shared_key)?;

        self.bridge
            .call3(&JsValue::NULL, &instruction, &psk_id, &psk)
            .map_err(InstructionError::JsError)?;

        Ok(())
    }

    fn mls_group_join_config<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        MlsGroupJoinConfig: openmls_traits::storage::traits::MlsGroupJoinConfig<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<MlsGroupJoinConfig>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadMlsGroupJoinConfig)?;

        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn own_leaf_nodes<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        LeafNode: openmls_traits::storage::traits::LeafNode<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Vec<LeafNode>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadOwnLeafNodes)?;

        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn queued_proposal_refs<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ProposalRef: openmls_traits::storage::traits::ProposalRef<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Vec<ProposalRef>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadQueuedProposalReferences)?;

        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn queued_proposals<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ProposalRef: openmls_traits::storage::traits::ProposalRef<VERSION>,
        QueuedProposal: openmls_traits::storage::traits::QueuedProposal<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Vec<(ProposalRef, QueuedProposal)>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadQueuedProposals)?;

        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn tree<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        TreeSync: openmls_traits::storage::traits::TreeSync<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<TreeSync>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadTree)?;

        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn group_context<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        GroupContext: openmls_traits::storage::traits::GroupContext<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<GroupContext>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadGroupContext)?;

        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn interim_transcript_hash<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        InterimTranscriptHash: openmls_traits::storage::traits::InterimTranscriptHash<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<InterimTranscriptHash>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadInterimTranscriptHash)?;

        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn confirmation_tag<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ConfirmationTag: openmls_traits::storage::traits::ConfirmationTag<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<ConfirmationTag>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadConfirmationTag)?;

        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn group_state<
        GroupState: openmls_traits::storage::traits::GroupState<VERSION>,
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<GroupState>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadGroupState)?;

        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn message_secrets<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        MessageSecrets: openmls_traits::storage::traits::MessageSecrets<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<MessageSecrets>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadMessageSecrets)?;

        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn resumption_psk_store<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ResumptionPskStore: openmls_traits::storage::traits::ResumptionPskStore<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<ResumptionPskStore>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadResumptionPskStore)?;

        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn own_leaf_index<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        LeafNodeIndex: openmls_traits::storage::traits::LeafNodeIndex<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<LeafNodeIndex>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadOwnLeafIndex)?;

        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn group_epoch_secrets<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        GroupEpochSecrets: openmls_traits::storage::traits::GroupEpochSecrets<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<Option<GroupEpochSecrets>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadGroupEpochSecrets)?;

        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn signature_key_pair<
        SignaturePublicKey: openmls_traits::storage::traits::SignaturePublicKey<VERSION>,
        SignatureKeyPair: openmls_traits::storage::traits::SignatureKeyPair<VERSION>,
    >(
        &self,
        public_key: &SignaturePublicKey,
    ) -> Result<Option<SignatureKeyPair>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadSignatureKeyPair)?;

        let public_key = serde_wasm_bindgen::to_value(public_key)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &public_key)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn encryption_key_pair<
        HpkeKeyPair: openmls_traits::storage::traits::HpkeKeyPair<VERSION>,
        EncryptionKey: openmls_traits::storage::traits::EncryptionKey<VERSION>,
    >(
        &self,
        public_key: &EncryptionKey,
    ) -> Result<Option<HpkeKeyPair>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadEncryptionKeyPair)?;

        let public_key = serde_wasm_bindgen::to_value(public_key)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &public_key)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
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
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadEncryptionEpochKeyPairs)?;

        let group_id = serde_wasm_bindgen::to_value(group_id)?;
        let epoch = serde_wasm_bindgen::to_value(epoch)?;
        let leaf_index = serde_wasm_bindgen::to_value(&leaf_index)?;
        let keys = js_sys::Array::of3(&group_id, &epoch, &leaf_index);

        let result = self
            .bridge
            .call3(&JsValue::NULL, &instruction, &keys, &epoch)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn key_package<
        KeyPackageRef: openmls_traits::storage::traits::HashReference<VERSION>,
        KeyPackage: openmls_traits::storage::traits::KeyPackage<VERSION>,
    >(
        &self,
        hash_ref: &KeyPackageRef,
    ) -> Result<Option<KeyPackage>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadKeyPackage)?;
        let hash_ref = serde_wasm_bindgen::to_value(hash_ref)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &hash_ref)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn psk<
        PskBundle: openmls_traits::storage::traits::PskBundle<VERSION>,
        PskId: openmls_traits::storage::traits::PskId<VERSION>,
    >(
        &self,
        pre_shared_key_id: &PskId,
    ) -> Result<Option<PskBundle>, Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ReadPreSharedKey)?;
        let pre_shared_key_id = serde_wasm_bindgen::to_value(pre_shared_key_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &pre_shared_key_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn remove_proposal<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ProposalRef: openmls_traits::storage::traits::ProposalRef<VERSION>,
    >(
        &self,
        group_id: &GroupId,
        proposal_reference: &ProposalRef,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::RemoveProposal)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;
        let proposal_reference = serde_wasm_bindgen::to_value(proposal_reference)?;
        let keys = js_sys::Array::of2(&group_id, &proposal_reference);

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &keys)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_own_leaf_nodes<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeleteOwnLeafNodes)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_group_config<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeleteGroupConfiguration)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_tree<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeleteTree)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_confirmation_tag<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeleteConfirmationTag)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_group_state<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeleteGroupState)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_context<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeleteContext)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_interim_transcript_hash<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeleteInterimTranscriptHash)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_message_secrets<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeleteMessageSecrets)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_all_resumption_psk_secrets<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let instruction =
            serde_wasm_bindgen::to_value(&Operation::DeleteAllResumptionPreSharedKeySecrets)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_own_leaf_index<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeleteOwnLeafIndex)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_group_epoch_secrets<GroupId: openmls_traits::storage::traits::GroupId<VERSION>>(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeleteGroupEpochSecrets)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn clear_proposal_queue<
        GroupId: openmls_traits::storage::traits::GroupId<VERSION>,
        ProposalRef: openmls_traits::storage::traits::ProposalRef<VERSION>,
    >(
        &self,
        group_id: &GroupId,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::ClearProposalQueue)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &group_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_signature_key_pair<
        SignaturePublicKey: openmls_traits::storage::traits::SignaturePublicKey<VERSION>,
    >(
        &self,
        public_key: &SignaturePublicKey,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeleteSignatureKeyPair)?;
        let public_key = serde_wasm_bindgen::to_value(public_key)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &public_key)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_encryption_key_pair<
        EncryptionKey: openmls_traits::storage::traits::EncryptionKey<VERSION>,
    >(
        &self,
        public_key: &EncryptionKey,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeleteEncryptionKeyPair)?;
        let public_key = serde_wasm_bindgen::to_value(public_key)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &public_key)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
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
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeleteEncryptionEpochKeyPairs)?;
        let group_id = serde_wasm_bindgen::to_value(group_id)?;
        let epoch = serde_wasm_bindgen::to_value(epoch)?;
        let leaf_index = serde_wasm_bindgen::to_value(&leaf_index)?;
        let keys = js_sys::Array::of3(&group_id, &epoch, &leaf_index);

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &keys)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_key_package<
        KeyPackageRef: openmls_traits::storage::traits::HashReference<VERSION>,
    >(
        &self,
        hash_reference: &KeyPackageRef,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeleteKeyPackage)?;
        let hash_reference = serde_wasm_bindgen::to_value(hash_reference)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &hash_reference)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }

    fn delete_psk<PskKey: openmls_traits::storage::traits::PskId<VERSION>>(
        &self,
        psk_id: &PskKey,
    ) -> Result<(), Self::Error> {
        let instruction = serde_wasm_bindgen::to_value(&Operation::DeletePsk)?;
        let psk_id = serde_wasm_bindgen::to_value(psk_id)?;

        let result = self
            .bridge
            .call2(&JsValue::NULL, &instruction, &psk_id)
            .map_err(InstructionError::JsError)?;

        serde_wasm_bindgen::from_value(result).map_err(InstructionError::Deserialize)
    }
}
