use cosmwasm_std::{CosmosMsg, Uint64};
use cw_orch::prelude::*;
use cw_orch_interchain::{IbcQueryHandler, InterchainEnv, InterchainError};
use polytone_note::msg::ExecuteMsgFns;

use crate::{
    deploy::{POLYTONE_NOTE, POLYTONE_PROXY, POLYTONE_VOICE},
    Polytone, PolytoneNote, PolytoneProxy, PolytoneVoice,
};

/// Represents an Polytone connection
///
/// The note contract is on the local chain while the voice and proxy contracts are located on the remote chain
#[derive(Clone)]
pub struct PolytoneConnection<Chain: CwEnv> {
    pub note: PolytoneNote<Chain>,
    pub voice: PolytoneVoice<Chain>,
    pub proxy: PolytoneProxy<Chain>, // This contract doesn't have an address, it's only a code id  used for instantiating
}

impl<Chain: CwEnv> PolytoneConnection<Chain> {
    pub fn load_from(src_chain: Chain, dst_chain: Chain) -> PolytoneConnection<Chain> {
        PolytoneConnection {
            note: PolytoneNote::new(
                format!("{} | {}", POLYTONE_NOTE, dst_chain.env_info().chain_id),
                src_chain.clone(),
            ),
            voice: PolytoneVoice::new(
                format!("{} | {}", POLYTONE_VOICE, src_chain.env_info().chain_id),
                dst_chain.clone(),
            ),
            // Proxy doesn't have a specific address in deployments, so we don't load a specific suffixed proxy
            proxy: PolytoneProxy::new(POLYTONE_PROXY, dst_chain),
        }
    }

    pub fn send_message(&self, msgs: Vec<CosmosMsg>) -> Result<Chain::Response, CwOrchError> {
        self.note
            .ibc_execute(msgs, Uint64::from(1_000_000u64), None)
    }
}

impl<Chain: IbcQueryHandler> PolytoneConnection<Chain> {
    pub fn deploy_between(
        interchain: &impl InterchainEnv<Chain>,
        src_chain_id: &str,
        dst_chain_id: &str,
    ) -> Result<Self, InterchainError> {
        let src_chain = interchain.chain(src_chain_id).map_err(Into::into)?;
        let dst_chain = interchain.chain(dst_chain_id).map_err(Into::into)?;
        let src_polytone = Polytone::store_on(src_chain)?;
        let dst_polytone = Polytone::store_on(dst_chain)?;

        src_polytone.connect(&dst_polytone, interchain)
    }

    pub fn deploy_between_if_needed(
        interchain: &impl InterchainEnv<Chain>,
        src_chain_id: &str,
        dst_chain_id: &str,
    ) -> Result<Self, InterchainError> {
        let src_chain = interchain.chain(src_chain_id).map_err(Into::into)?;
        let dst_chain = interchain.chain(dst_chain_id).map_err(Into::into)?;

        let src_polytone = Polytone::store_if_needed(src_chain)?;
        let dst_polytone = Polytone::store_if_needed(dst_chain)?;

        src_polytone.connect_if_needed(&dst_polytone, interchain)
    }
}
