use core::str::FromStr;

use syn::{Signature, Type};

use crate::common::context::CodegenContext;

use super::{Accepts, AcceptsBuilder, AcceptsInfo, AsyncAccepts, DynAsyncAccepts};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AcceptsEnum {
    Sync(Accepts),
    Async(AsyncAccepts),
    Dyn(DynAsyncAccepts),
}

impl AcceptsInfo for AcceptsEnum {
    fn accepts_name(&self) -> &'static str {
        match self {
            Self::Sync(accepts) => accepts.accepts_name(),
            Self::Async(async_accepts) => async_accepts.accepts_name(),
            Self::Dyn(dyn_accepts) => dyn_accepts.accepts_name(),
        }
    }

    fn accept_fn_name(&self) -> &'static str {
        match self {
            Self::Sync(accepts) => accepts.accept_fn_name(),
            Self::Async(async_accepts) => async_accepts.accept_fn_name(),
            Self::Dyn(dyn_accepts) => dyn_accepts.accept_fn_name(),
        }
    }

    fn accept_lifetime_name(&self) -> Option<&'static str> {
        match self {
            Self::Sync(accepts) => accepts.accept_lifetime_name(),
            Self::Async(async_accepts) => async_accepts.accept_lifetime_name(),
            Self::Dyn(dyn_accepts) => dyn_accepts.accept_lifetime_name(),
        }
    }

    fn accept_is_async(&self) -> bool {
        match self {
            Self::Sync(accepts) => accepts.accept_is_async(),
            Self::Async(async_accepts) => async_accepts.accept_is_async(),
            Self::Dyn(dyn_accepts) => dyn_accepts.accept_is_async(),
        }
    }
}
impl AcceptsBuilder for AcceptsEnum {
    fn build_accept_signature(&self, ctx: &CodegenContext, accepts_t_type: Type) -> Signature {
        match self {
            Self::Sync(accepts) => accepts.build_accept_signature(ctx, accepts_t_type),
            Self::Async(async_accepts) => async_accepts.build_accept_signature(ctx, accepts_t_type),
            Self::Dyn(dyn_accepts) => dyn_accepts.build_accept_signature(ctx, accepts_t_type),
        }
    }
}

impl FromStr for AcceptsEnum {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Ok(accepts) = Accepts::from_str(s) {
            Ok(Self::Sync(accepts))
        } else if let Ok(async_accepts) = AsyncAccepts::from_str(s) {
            Ok(Self::Async(async_accepts))
        } else if let Ok(dyn_accepts) = DynAsyncAccepts::from_str(s) {
            Ok(Self::Dyn(dyn_accepts))
        } else {
            Err(())
        }
    }
}
