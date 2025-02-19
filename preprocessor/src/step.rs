// The Licensed Work is (c) 2023 ChainSafe
// Code: https://github.com/ChainSafe/Spectre
// SPDX-License-Identifier: LGPL-3.0-only
use beacon_api_client::Client;
use beacon_api_client::{BlockId, ClientTypes, StateId};
use eth_types::Spec;
use ethereum_consensus_types::bls::BlsPublicKey;
use ethereum_consensus_types::signing::{compute_domain, DomainType};
use ethereum_consensus_types::{ForkData, LightClientBootstrap, LightClientFinalityUpdate};
use itertools::Itertools;
use ssz_rs::Vector;
use ssz_rs::{Merkleized, Node};
use step_iso::types::{BeaconBlockHeader, SyncStepArgs};

use crate::{get_light_client_bootstrap, get_light_client_finality_update};
/// Fetches the latest `LightClientFinalityUpdate`` and the current sync committee (from LightClientBootstrap) and converts it to a [`SyncStepArgs`] witness.
pub async fn fetch_step_args<S: Spec, C: ClientTypes>(
    client: &Client<C>,
) -> eyre::Result<SyncStepArgs>
where
    [(); S::SYNC_COMMITTEE_SIZE]:,
    [(); S::FINALIZED_HEADER_DEPTH]:,
    [(); S::SYNC_COMMITTEE_DEPTH]:,
    [(); S::BYTES_PER_LOGS_BLOOM]:,
    [(); S::MAX_EXTRA_DATA_BYTES]:,
{
    let finality_update = get_light_client_finality_update(client).await?;
    let block_root = client
        .get_beacon_block_root(BlockId::Slot(finality_update.finalized_header.beacon.slot))
        .await
        .unwrap();
    let bootstrap: LightClientBootstrap<
        { S::SYNC_COMMITTEE_SIZE },
        { S::SYNC_COMMITTEE_DEPTH },
        { S::BYTES_PER_LOGS_BLOOM },
        { S::MAX_EXTRA_DATA_BYTES },
    > = get_light_client_bootstrap(client, block_root).await?;
    let pubkeys_compressed = bootstrap.current_sync_committee.pubkeys;
    let attested_state_id = finality_update.attested_header.beacon.state_root;
    let fork_version = client
        .get_fork(StateId::Root(attested_state_id))
        .await?
        .current_version;
    let genesis_validators_root = client.get_genesis_details().await?.genesis_validators_root;
    let fork_data = ForkData {
        genesis_validators_root,
        fork_version,
    };
    let domain = compute_domain(DomainType::SyncCommittee, &fork_data)?;

    step_args_from_finality_update(finality_update, pubkeys_compressed, domain).await
}

/// Converts a [`LightClientFinalityUpdate`] to a [`SyncStepArgs`] witness.
pub async fn step_args_from_finality_update<S: Spec>(
    finality_update: LightClientFinalityUpdate<
        { S::SYNC_COMMITTEE_SIZE },
        { S::FINALIZED_HEADER_DEPTH },
        { S::BYTES_PER_LOGS_BLOOM },
        { S::MAX_EXTRA_DATA_BYTES },
    >,
    pubkeys_compressed: Vector<BlsPublicKey, { S::SYNC_COMMITTEE_SIZE }>,
    domain: [u8; 32],
) -> eyre::Result<SyncStepArgs> {
    let pubkeys_uncompressed = pubkeys_compressed
        .iter()
        .map(|pk| pk.decompressed_bytes())
        .collect_vec();

    let execution_payload_root = finality_update
        .finalized_header
        .execution
        .clone()
        .hash_tree_root()?
        .to_vec();

    let execution_payload_branch = finality_update
        .finalized_header
        .execution_branch
        .iter()
        .map(|n| n.0.to_vec())
        .collect_vec();

    assert!(
        ssz_rs::is_valid_merkle_branch(
            Node::try_from(execution_payload_root.as_slice())?,
            &execution_payload_branch,
            S::EXECUTION_STATE_ROOT_DEPTH,
            S::EXECUTION_STATE_ROOT_INDEX,
            finality_update.finalized_header.beacon.body_root,
        )
        .is_ok(),
        "Execution payload merkle proof verification failed"
    );

    assert!(
        ssz_rs::is_valid_merkle_branch(
            finality_update
                .finalized_header
                .beacon
                .clone()
                .hash_tree_root()
                .unwrap(),
            &finality_update
                .finality_branch
                .iter()
                .map(|n| n.as_ref())
                .collect_vec(),
            S::FINALIZED_HEADER_DEPTH,
            S::FINALIZED_HEADER_INDEX,
            finality_update.attested_header.beacon.state_root,
        )
        .is_ok(),
        "Finality merkle proof verification failed"
    );

    Ok(SyncStepArgs {
        signature_compressed: finality_update
            .sync_aggregate
            .sync_committee_signature
            .to_bytes()
            .to_vec(),
        pubkeys_uncompressed,
        pariticipation_bits: finality_update
            .sync_aggregate
            .sync_committee_bits
            .iter()
            .by_vals()
            .collect_vec(),
        attested_header: step_iso::types::BeaconBlockHeader {
            slot: finality_update.attested_header.beacon.slot.to_string(),
            proposer_index: finality_update
                .attested_header
                .beacon
                .proposer_index
                .to_string(),
            parent_root: finality_update.attested_header.beacon.parent_root,
            state_root: finality_update.attested_header.beacon.state_root,
            body_root: finality_update.attested_header.beacon.body_root,
        },
        finalized_header: BeaconBlockHeader {
            slot: finality_update.finalized_header.beacon.slot.to_string(),
            proposer_index: finality_update
                .finalized_header
                .beacon
                .proposer_index
                .to_string(),
            parent_root: finality_update.finalized_header.beacon.parent_root,
            state_root: finality_update.finalized_header.beacon.state_root,
            body_root: finality_update.finalized_header.beacon.body_root,
        },
        finality_branch: finality_update
            .finality_branch
            .iter()
            .map(|n| n.0.to_vec())
            .collect_vec(),
        execution_payload_root: finality_update
            .finalized_header
            .execution
            .clone()
            .hash_tree_root()
            .unwrap()
            .to_vec(),
        execution_payload_branch: finality_update
            .finalized_header
            .execution_branch
            .iter()
            .map(|n| n.0.to_vec())
            .collect_vec(),
        domain,
    })
}
