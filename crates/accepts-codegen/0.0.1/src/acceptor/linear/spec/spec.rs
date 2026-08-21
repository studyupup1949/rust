use syn::{Attribute, Ident, TypeParam, Visibility};

use super::{HandlerSpec, NextAcceptorSpec, config::AcceptsImplsConfig};

#[derive(Debug, Clone)]
pub struct LinearAcceptorSpec {
    pub vis: Visibility,
    pub ident: Ident,
    pub attrs: Vec<Attribute>,
    pub accepts_value_param: TypeParam,
    pub accept_impls: AcceptsImplsConfig,
    pub handler: HandlerSpec,
    pub next: Option<NextAcceptorSpec>,
}
