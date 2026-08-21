use ethers::prelude::*;
use ethers::abi::Abi;
use ethers::providers::{Provider, Http};
use std::sync::Arc;
use std::fs::File;
use std::io::Read;
use serde_json;

/// Creates a Contract instance using the provided ABI file and contract address.
/// 
/// # Arguments
/// * `abi_path` - Path to the ABI file.
/// * `contract_address` - Address of the contract.
/// * `provider` - Arc-wrapped Provider instance for interacting with the Ethereum network.
///
/// # Returns
/// A Contract instance that allows interaction with the contract deployed at the provided address.
pub fn create_access_manager_instance(
    abi_path: &str, 
    contract_address: &str, 
    provider: Arc<Provider<Http>>
) -> Result<Contract<Provider<Http>>, Box<dyn std::error::Error>> {
    // Read the ABI file
    let mut abi_file = File::open(abi_path)?;
    let mut abi_content = String::new();
    abi_file.read_to_string(&mut abi_content)?;

    // Deserialize the ABI content into an `Abi` structure
    let abi: Abi = serde_json::from_str(&abi_content)?;

    // Parse the contract address and create the Contract instance
    let contract_address: Address = contract_address.parse()?;
    let contract = Contract::new(contract_address, abi, provider);

    Ok(contract)
}
