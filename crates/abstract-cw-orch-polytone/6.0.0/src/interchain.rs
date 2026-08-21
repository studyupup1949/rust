use cosmwasm_std::{CosmosMsg, Uint64};
use cw_orch::{core::serde_json::Value, daemon::DeployedChains, prelude::*};
use cw_orch_interchain::core::{IbcQueryHandler, InterchainEnv, InterchainError};
use polytone_note::msg::ExecuteMsgFns;

use crate::{
    deploy::{POLYTONE_NOTE, POLYTONE_PROXY, POLYTONE_VOICE},
    utils::read_json,
    Polytone, PolytoneNote, PolytoneProxy, PolytoneVoice,
};

pub const DELIMITER: &str = " | ";

/// Represents an Polytone connection
///
/// The note contract is on the local chain while the voice and proxy contracts are located on the remote chain
#[derive(Clone)]
pub struct PolytoneConnection<Chain: CwEnv> {
    pub note: PolytoneNote<Chain>,
    pub voice: PolytoneVoice<Chain>,
    pub proxy: PolytoneProxy<Chain>, // This contract doesn't have an address, it's only a code id  used for instantiating
}

impl<Chain: IbcQueryHandler> PolytoneConnection<Chain> {
    pub fn load_from(src_chain: Chain, dst_chain: Chain) -> PolytoneConnection<Chain> {
        let mut connection = PolytoneConnection {
            note: PolytoneNote::new(
                format!(
                    "{POLYTONE_NOTE}{DELIMITER}{}",
                    dst_chain.env_info().chain_id
                ),
                src_chain.clone(),
            ),
            voice: PolytoneVoice::new(
                format!(
                    "{POLYTONE_VOICE}{DELIMITER}{}",
                    src_chain.env_info().chain_id
                ),
                dst_chain.clone(),
            ),
            // Proxy doesn't have a specific address in deployments, so we don't load a specific suffixed proxy
            proxy: PolytoneProxy::new(POLYTONE_PROXY, dst_chain),
        };

        connection.set_contracts_addresses();
        connection
    }

    /// This allows loading only the addresses from the state, because code_ids are not relevant for this Structure
    fn set_contracts_addresses(&mut self) {
        let state;

        let state_file = Polytone::<Chain>::deployed_state_file_path();
        if let Some(state_file) = state_file {
            if let Ok(module_state_json) = read_json(&state_file) {
                state = module_state_json;
            } else {
                return;
            }
        } else {
            return;
        }

        let all_contracts = self.get_contracts_mut();

        for contract in all_contracts {
            if !contract.environment().can_load_state_from_state_file() {
                continue;
            }
            // We set the code_id and/or address of the contract in question if they are not present already
            let env_info = contract.environment().env_info();
            // We try to get the address for the contract
            if contract.address().is_err() {
                // Try and get the code id from file
                let address = state
                    .get(env_info.chain_id.to_string())
                    .unwrap_or(&Value::Null)
                    .get(env_info.deployment_id)
                    .unwrap_or(&Value::Null)
                    .get(contract.id());

                if let Some(address) = address {
                    if address.is_string() {
                        contract.set_default_address(&Addr::unchecked(address.as_str().unwrap()))
                    }
                }
            }
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
        let src_chain = interchain.get_chain(src_chain_id).map_err(Into::into)?;
        let dst_chain = interchain.get_chain(dst_chain_id).map_err(Into::into)?;
        let src_polytone = Polytone::store_on(src_chain)?;
        let dst_polytone = Polytone::store_on(dst_chain)?;

        src_polytone.connect(&dst_polytone, interchain)
    }

    pub fn deploy_between_if_needed(
        interchain: &impl InterchainEnv<Chain>,
        src_chain_id: &str,
        dst_chain_id: &str,
    ) -> Result<Self, InterchainError> {
        let src_chain = interchain.get_chain(src_chain_id).map_err(Into::into)?;
        let dst_chain = interchain.get_chain(dst_chain_id).map_err(Into::into)?;

        let src_polytone = Polytone::store_if_needed(src_chain)?;
        let dst_polytone = Polytone::store_if_needed(dst_chain)?;

        src_polytone.connect_if_needed(&dst_polytone, interchain)
    }

    fn get_contracts_mut(&mut self) -> Vec<&mut dyn cw_orch::prelude::ContractInstance<Chain>> {
        vec![&mut self.note, &mut self.voice, &mut self.proxy]
    }
}
