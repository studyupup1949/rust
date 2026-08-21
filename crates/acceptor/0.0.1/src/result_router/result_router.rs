use accepts::Accepts;
use core::marker::PhantomData;

/// `Accepts<Result<OkValue, ErrValue>>` implementation that delegates to `Ok` or `Err` acceptors.
#[must_use = "ResultRouter must be used to forward results to the correct acceptor"]
#[derive(Debug, Clone)]
pub struct ResultRouter<OkValue, ErrValue, OkAccepts, ErrAccepts> {
    ok_acceptor: OkAccepts,
    err_acceptor: ErrAccepts,
    _marker: PhantomData<(OkValue, ErrValue)>,
}

impl<OkValue, ErrValue, OkAccepts, ErrAccepts>
    ResultRouter<OkValue, ErrValue, OkAccepts, ErrAccepts>
where
    OkAccepts: Accepts<OkValue>,
    ErrAccepts: Accepts<ErrValue>,
{
    /// Creates a new `ResultRouter`.
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

impl<OkValue, ErrValue, OkAccepts, ErrAccepts> Accepts<Result<OkValue, ErrValue>>
    for ResultRouter<OkValue, ErrValue, OkAccepts, ErrAccepts>
where
    OkAccepts: Accepts<OkValue>,
    ErrAccepts: Accepts<ErrValue>,
{
    fn accept(&self, value: Result<OkValue, ErrValue>) {
        match value {
            Ok(v) => self.ok_acceptor.accept(v),
            Err(e) => self.err_acceptor.accept(e),
        }
    }
}
