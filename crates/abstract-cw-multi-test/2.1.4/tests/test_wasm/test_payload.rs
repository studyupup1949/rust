use cosmwasm_std::{testing::MockApi, to_json_binary, Empty, WasmMsg};
use cw_multi_test::{no_init, AppBuilder, Executor};

use crate::test_contracts;

#[test]
fn test_payload() {
    // prepare application with custom API
    let mut app = AppBuilder::default()
        .with_api(MockApi::default().with_prefix("purple"))
        .build(no_init);

    // prepare user addresses
    let creator_addr = app.api().addr_make("creator");

    // store contract's code
    let code_id = app.store_code_with_creator(creator_addr, test_contracts::counter::contract());

    let owner = app.api().addr_make("owner");

    let contract_addr_1 = app
        .instantiate_contract(code_id, owner.clone(), &Empty {}, &[], "Counter", None)
        .unwrap();

    app.execute(
        owner,
        WasmMsg::Execute {
            contract_addr: contract_addr_1.to_string(),
            msg: to_json_binary(&WasmMsg::ClearAdmin {
                contract_addr: contract_addr_1.to_string(),
            })
            .unwrap(),
            funds: vec![],
        }
        .into(),
    )
    .unwrap();
}
