use abstract_os::objects::module::ModuleId;
use abstract_sdk::{Modules, ModuleInterface, AbstractSdkResult};
use cosmwasm_std::{Deps, Empty, CosmosMsg, wasm_execute};
use serde::{Serialize, de::DeserializeOwned};

use abstract_os::app as msg;

/// Interact with other modules on the OS.
pub trait AppInterface: ModuleInterface {
    fn app<'a>(&'a self, deps: Deps<'a>) -> App<Self> {
        App { base: self, deps }
    }
}

impl<T> AppInterface for T where T: ModuleInterface {}

#[derive(Clone)]
pub struct App<'a, T: AppInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: AppInterface> App<'a, T> {
    /// Construct an app request message.
    fn app_request<M: Serialize>(
        &self,
        app_id: ModuleId,
        message: impl Into<msg::ExecuteMsg<M, Empty>>,
    ) -> AbstractSdkResult<CosmosMsg> {
        let modules = self.base.modules(self.deps);
        modules.assert_module_dependency(app_id)?;
        let app_msg: msg::ExecuteMsg<M, Empty> = message.into();
        let app_address = modules.module_address(app_id)?;
        Ok(wasm_execute(app_address, &app_msg, vec![])?.into())
    }

    /// Construct an app configuation message
    fn app_configure(
        &self,
        app_id: ModuleId,
        message: msg::BaseExecuteMsg,
    ) -> AbstractSdkResult<CosmosMsg> {
        let app_msg: msg::ExecuteMsg<Empty, Empty> = message.into();
        let modules = self.base.modules(self.deps);
        let app_address = modules.module_address(app_id)?;
        Ok(wasm_execute(app_address, &app_msg, vec![])?.into())
    }

    /// Smart query an app
    fn query_app<Q: Serialize, R: DeserializeOwned>(
        &self,
        app_id: ModuleId,
        message: impl Into<msg::QueryMsg<Q>>,
    ) -> AbstractSdkResult<R> {
        let modules = self.base.modules(self.deps);
        let app_msg: msg::QueryMsg<Q> = message.into();
        let app_address = modules.module_address(app_id)?;
        self.deps
            .querier
            .query_wasm_smart(app_address, &app_msg)
            .map_err(Into::into)
    }
}