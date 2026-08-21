use core::{future::Future, marker::PhantomData};

use accepts_macros::auto_impl_dyn_internal;

use crate::core_traits::AsyncAccepts;

/// `Accepts<Result<OkValue, ErrValue>>` implementation that delegates to `Ok` or `Err` acceptors.
#[must_use = "ResultAsyncAcceptor must be used to route results to the correct async acceptor"]
#[derive(Debug, Clone)]
pub struct ResultAsyncAcceptor<OkValue, ErrValue, OkAccepts, ErrAccepts>
where
    OkAccepts: AsyncAccepts<OkValue>,
    ErrAccepts: AsyncAccepts<ErrValue>,
{
    ok_acceptor: OkAccepts,
    err_acceptor: ErrAccepts,
    _marker: PhantomData<(OkValue, ErrValue)>,
}

impl<OkValue, ErrValue, OkAccepts, ErrAccepts>
    ResultAsyncAcceptor<OkValue, ErrValue, OkAccepts, ErrAccepts>
where
    OkAccepts: AsyncAccepts<OkValue>,
    ErrAccepts: AsyncAccepts<ErrValue>,
{
    /// Creates a new `ResultAsyncAcceptor`.
    pub fn new(ok_acceptor: OkAccepts, err_acceptor: ErrAccepts) -> Self {
        Self {
            ok_acceptor,
            err_acceptor,
            _marker: PhantomData,
        }
    }

    pub fn ok_acceptor(&self) -> &OkAccepts {
        &self.ok_acceptor
    }

    pub fn ok_acceptor_mut(&mut self) -> &mut OkAccepts {
        &mut self.ok_acceptor
    }

    pub fn err_acceptor(&self) -> &ErrAccepts {
        &self.err_acceptor
    }

    pub fn err_acceptor_mut(&mut self) -> &mut ErrAccepts {
        &mut self.err_acceptor
    }
}

#[auto_impl_dyn_internal(cfg(feature = "alloc"))]
impl<OkValue, ErrValue, OkAccepts, ErrAccepts> AsyncAccepts<Result<OkValue, ErrValue>>
    for ResultAsyncAcceptor<OkValue, ErrValue, OkAccepts, ErrAccepts>
where
    OkAccepts: AsyncAccepts<OkValue>,
    ErrAccepts: AsyncAccepts<ErrValue>,
{
    fn accept_async<'a>(&'a self, value: Result<OkValue, ErrValue>) -> impl Future<Output = ()> + 'a
    where
        Result<OkValue, ErrValue>: 'a,
    {
        async {
            match value {
                Ok(v) => self.ok_acceptor.accept_async(v).await,
                Err(e) => self.err_acceptor.accept_async(e).await,
            }
        }
    }
}
