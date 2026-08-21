use syn::{Attribute, Ident, Path, Type, TypeParam, TypePath, Visibility};

use crate::{
    acceptor::linear::config::HandlerConfig2,
    common::{function::generate::option_path, syn::ext::TypePathConstructExt},
};

use super::{
    AcceptsImplsConfig, HandlerConfig, NextAcceptorConfig,
    spec::{
        HandlerErrorAcceptorSpec, HandlerSpec, LinearAcceptorSpec, MutGuardErrorAcceptorSpec,
        MutGuardSpec, NextAcceptorSpec,
    },
};

/// Builder for `LinearAcceptorSpec`.
#[derive(Debug, Clone, Default)]
pub struct LinearAcceptorConfig {
    vis: Option<Visibility>,
    ident: Option<Ident>,
    attrs: Vec<Attribute>,
    accepts_value_param: Option<TypeParam>,

    accept_impls: Option<AcceptsImplsConfig>,

    next_accepts: Option<Option<NextAcceptorConfig>>,

    handler: Option<HandlerConfig>,
}

#[allow(dead_code)]
impl LinearAcceptorConfig {
    /// Create an empty builder.
    pub fn new() -> Self {
        Self::default()
    }

    /// Set parsed attributes.
    pub fn attrs(&mut self, attributes: Vec<Attribute>) -> &mut Self {
        self.attrs = attributes;
        self
    }

    /// Set visibility of the generated type.
    pub fn vis(&mut self, visibility: Visibility) -> &mut Self {
        self.vis = Some(visibility);
        self
    }

    /// Set identifier of the generated type.
    pub fn ident(&mut self, name: Ident) -> &mut Self {
        self.ident = Some(name);
        self
    }

    /// Set the accepts type parameter.
    pub fn accepts_value_param(&mut self, accepts_value_param: TypeParam) -> &mut Self {
        self.accepts_value_param = Some(accepts_value_param);
        self
    }

    /// Set accept implementation generation configuration.
    pub fn accept_impls(&mut self, gen_accepts_type: AcceptsImplsConfig) -> &mut Self {
        self.accept_impls = Some(gen_accepts_type);
        self
    }

    /// Set continuation configuration.
    pub fn next_accepts(&mut self, next_accepts: Option<NextAcceptorConfig>) -> &mut Self {
        self.next_accepts = Some(next_accepts);
        self
    }

    /// Set handler configuration.
    pub fn handler(&mut self, handler: HandlerConfig) -> &mut Self {
        self.handler = Some(handler);
        self
    }

    /// Get the parsed attributes.
    pub fn get_attrs(&self) -> &[Attribute] {
        &self.attrs
    }

    /// Get visibility of the generated type.
    pub fn get_vis(&self) -> Option<&Visibility> {
        self.vis.as_ref()
    }

    /// Get identifier of the generated type.
    pub fn get_ident(&self) -> Option<&Ident> {
        self.ident.as_ref()
    }

    /// Get the accepts type parameter.
    pub fn get_accepts_value_param(&self) -> Option<&TypeParam> {
        self.accepts_value_param.as_ref()
    }

    /// Get configuration for accept implementations.
    pub fn get_accept_impls(&self) -> Option<&AcceptsImplsConfig> {
        self.accept_impls.as_ref()
    }

    /// Get continuation configuration.
    pub fn get_next_accepts(&self) -> Option<&Option<NextAcceptorConfig>> {
        self.next_accepts.as_ref()
    }

    /// Get handler configuration.
    pub fn get_handler(&self) -> Option<&HandlerConfig> {
        self.handler.as_ref()
    }

    /// Build the final `LinearAcceptorSpec`.
    pub fn build_spec(self) -> syn::Result<LinearAcceptorSpec> {
        let vis = self.vis.ok_or_else(|| {
            syn::Error::new(proc_macro2::Span::call_site(), "`visibility` is required")
        })?;
        let ident = self
            .ident
            .ok_or_else(|| syn::Error::new(proc_macro2::Span::call_site(), "`name` is required"))?;
        let accepts_value_param = self.accepts_value_param.ok_or_else(|| {
            syn::Error::new(proc_macro2::Span::call_site(), "`accepts_t` is required")
        })?;

        let accept_impls = self.accept_impls.ok_or_else(|| {
            syn::Error::new(
                proc_macro2::Span::call_site(),
                "`gen_accepts_type` is required",
            )
        })?;
        if accept_impls.is_empty() {
            return Err(syn::Error::new(
                proc_macro2::Span::call_site(),
                "`accepts_impls` must specify at least one of `Sync` or `Async`",
            ));
        }

        let next_accepts = self.next_accepts.ok_or_else(|| {
            syn::Error::new(proc_macro2::Span::call_site(), "`next_accepts` is required")
        })?;

        let handler = self.handler.ok_or_else(|| {
            syn::Error::new(proc_macro2::Span::call_site(), "`handler` is required")
        })?;

        let source_field_ident = handler.source_field_ident.clone();

        let (state_ty, mut_guard, source_ty) = match &handler.handler {
            HandlerConfig2::Ref { source_type } => {
                let (state_ty, source_ty) = source_type
                    .clone()
                    .into_state_source(accepts_value_param.ident.clone());
                (state_ty, None, source_ty)
            }
            HandlerConfig2::Mut {
                source_type,
                mut_guard: guard_access,
            } => {
                let (state_ty, guarded_ty, source_ty) = source_type
                    .clone()
                    .into_state_guarded_source(accepts_value_param.ident.clone());

                let guard_error = guard_access.guard_access_error.as_ref().map(|ge| {
                    let field_ty = Type::Path(TypePath::from_path(Path::from(
                        ge.accepts_generics_ident.clone(),
                    )));
                    MutGuardErrorAcceptorSpec {
                        generics_ident: ge.accepts_generics_ident.clone(),
                        field_ident: ge.accepts_field_ident.clone(),
                        field_ty,
                        error_ty: ge.error_ty.clone(),
                        forward_source: ge.forward_source.clone(),
                    }
                });
                let guard_spec = MutGuardSpec {
                    guard_path: guard_access.guard_access_path.clone(),
                    guarded_ty,
                    guard_error,
                };
                (state_ty, Some(guard_spec), source_ty)
            }
        };

        let next = next_accepts.map(|next| {
            let field_ty = Type::Path(TypePath::from_path(option_path(Some(Type::Path(
                TypePath::from_path(Path::from(next.next_accepts_generics_ident.clone())),
            )))));
            let accepts_ty = Type::Path(TypePath::from_path(Path::from(
                accepts_value_param.ident.clone(),
            )));
            NextAcceptorSpec {
                generics_ident: next.next_accepts_generics_ident.clone(),
                field_ident: next.next_acceptor_field_ident.clone(),
                field_ty,
                accepts_ty,
            }
        });

        let handler_error = handler.handler_error.as_ref().map(|he| {
            let field_ty = Type::Path(TypePath::from_path(Path::from(
                he.accepts_generics_ident.clone(),
            )));
            HandlerErrorAcceptorSpec {
                generics_ident: he.accepts_generics_ident.clone(),
                field_ident: he.accepts_field_ident.clone(),
                field_ty,
                context_ty: he.error_handler_ty.clone(),
            }
        });

        let handler = HandlerSpec {
            source_field_ident,
            source_ty,
            source_state_ty: state_ty,
            handler_path: handler.handler_path,
            handler_error,
            mut_guard,
        };

        Ok(LinearAcceptorSpec {
            vis,
            ident,
            attrs: self.attrs,
            accepts_value_param,
            accept_impls,
            handler,
            next,
        })
    }
}
