#![allow(dead_code)]

use abstract_os::{objects::module::ModuleId, api::ApiRequestMsg};
use abstract_sdk::{ModuleInterface, AbstractSdkResult};
use cosmwasm_std::{Empty, Deps, wasm_execute, CosmosMsg};
use serde::{Serialize, de::DeserializeOwned};

/// Interact with other modules on the OS.
pub trait ApiInterface: ModuleInterface {
    fn api<'a>(&'a self, deps: Deps<'a>) -> Api<Self> {
        Api { base: self, deps }
    }
}

impl<T> ApiInterface for T where T: ModuleInterface {}

#[derive(Clone)]
pub struct Api<'a, T: ApiInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: ApiInterface> Api <'a, T> {
    /// Interactions with Abstract APIs
    /// Construct an api request message.
    fn api_request<M: Serialize + Into<abstract_os::api::ExecuteMsg<M, Empty>>>(
        &self,
        api_id: ModuleId,
        message: M,
    ) -> AbstractSdkResult<CosmosMsg> {
        let modules = self.base.modules(self.deps);
        modules.assert_module_dependency(api_id)?;
        let api_msg = abstract_os::api::ExecuteMsg::<_>::App(ApiRequestMsg::new(
            Some(self.base.proxy_address(self.deps)?.into_string()),
            message,
        ));
        let api_address = modules.module_address(api_id)?;
        Ok(wasm_execute(api_address, &api_msg, vec![])?.into())
    }

    /// Smart query an API
    fn query_api<Q: Serialize, R: DeserializeOwned>(
        &self,
        api_id: ModuleId,
        message: impl Into<abstract_os::api::QueryMsg<Q>>,
    ) -> AbstractSdkResult<R> {
        let api_msg: abstract_os::api::QueryMsg<Q> = message.into();
        let modules = self.base.modules(self.deps);
        let api_address = modules.module_address(api_id)?;
        self.deps
            .querier
            .query_wasm_smart(api_address, &api_msg)
            .map_err(Into::into)
    }
}