#[cfg(any(feature = "internal-codegen", feature = "codegen"))]
use accepts_codegen::acceptor::{accepts_auto_impl, linear, next_acceptors_auto_impl};

#[cfg(feature = "internal-codegen")]
use accepts_codegen::common::context::INTERNAL_CONTEXT;

#[cfg(feature = "codegen")]
use accepts_codegen::common::context::PUBLIC_CONTEXT;

//===============linear_accepts===============
#[cfg(feature = "internal-codegen")]
#[proc_macro]
pub fn generate_linear_acceptor_internal(
    input: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    linear::expand(&INTERNAL_CONTEXT, input.into()).into()
}

#[cfg(feature = "codegen")]
#[proc_macro]
pub fn generate_linear_acceptor(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    linear::expand(&PUBLIC_CONTEXT, input.into()).into()
}

//===============auto_impl===============

#[cfg(feature = "internal-codegen")]
#[proc_macro_attribute]
pub fn auto_impl_async_internal(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    accepts_auto_impl::expand_async_impl(&INTERNAL_CONTEXT, attr.into(), item.into()).into()
}

#[cfg(feature = "codegen")]
#[proc_macro_attribute]
pub fn auto_impl_async(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    accepts_auto_impl::expand_async_impl(&PUBLIC_CONTEXT, attr.into(), item.into()).into()
}

#[cfg(feature = "internal-codegen")]
#[proc_macro_attribute]
pub fn auto_impl_dyn_internal(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    accepts_auto_impl::expand_dyn_impl(&INTERNAL_CONTEXT, attr.into(), item.into()).into()
}

#[cfg(feature = "codegen")]
#[proc_macro_attribute]
pub fn auto_impl_dyn(
    attr: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    accepts_auto_impl::expand_dyn_impl(&PUBLIC_CONTEXT, attr.into(), item.into()).into()
}

//===============derive NextAcceptors===============

#[cfg(feature = "internal-codegen")]
#[proc_macro_derive(NextAcceptorsInternal, attributes(next_acceptor))]
pub fn next_acceptors_internal(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    next_acceptors_auto_impl::expand(&INTERNAL_CONTEXT, item.into()).into()
}

#[cfg(feature = "codegen")]
#[proc_macro_derive(NextAcceptors, attributes(next_acceptor))]
pub fn next_acceptors(item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    next_acceptors_auto_impl::expand(&PUBLIC_CONTEXT, item.into()).into()
}
