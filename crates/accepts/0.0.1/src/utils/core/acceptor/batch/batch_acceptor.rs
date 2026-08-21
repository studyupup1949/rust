use core::{cell::RefCell, marker::PhantomData};

use crate::{core_traits::Accepts, macros::internal::codegen::NextAcceptorsInternal};

/// An `Accepts` implementation that batches values and
/// forwards them when the buffer reaches the specified size.
#[must_use = "BatchAcceptor must be used to buffer and flush values correctly"]
#[derive(Debug, Clone, NextAcceptorsInternal)]
pub struct BatchAcceptor<Value, Buffer, ShouldFlush, NextAccepts>
where
    Buffer: Extend<Value> + Default,
    ShouldFlush: Fn(&Buffer, &Value) -> bool,
    NextAccepts: Accepts<Buffer>,
{
    buffer: RefCell<Buffer>,
    #[next_acceptor(once, ref, mut)]
    next_acceptor: NextAccepts,
    should_flush: ShouldFlush,
    _marker: PhantomData<Value>,
}

impl<Value, Buffer, ShouldFlush, NextAccepts> BatchAcceptor<Value, Buffer, ShouldFlush, NextAccepts>
where
    Buffer: Extend<Value> + Default,
    ShouldFlush: Fn(&Buffer, &Value) -> bool,
    NextAccepts: Accepts<Buffer>,
{
    pub fn new(should_flush: ShouldFlush, next_acceptor: NextAccepts) -> Self {
        Self::with_buffer(Buffer::default(), should_flush, next_acceptor)
    }

    pub fn with_buffer(
        buffer: Buffer,
        should_flush: ShouldFlush,
        next_acceptor: NextAccepts,
    ) -> Self {
        Self {
            buffer: RefCell::new(buffer),
            next_acceptor,
            should_flush,
            _marker: PhantomData,
        }
    }

    /// Flushes any remaining values synchronously if the buffer is not empty.
    pub fn flush(&self)
    where
        for<'a> &'a Buffer: IntoIterator<Item = &'a Value>,
    {
        let mut buffer = self.buffer.borrow_mut();

        if (&*buffer).into_iter().next().is_none() {
            return;
        }

        let batch = core::mem::take(&mut *buffer);
        drop(buffer);
        self.next_acceptor.accept(batch);
    }

    /// Flushes any remaining values synchronously.
    pub fn force_flush(&self) {
        let batch = {
            let mut buffer = self.buffer.borrow_mut();
            core::mem::take(&mut *buffer)
        };
        self.next_acceptor.accept(batch);
    }
}

impl<Value, Buffer, ShouldFlush, NextAccepts> Accepts<Value>
    for BatchAcceptor<Value, Buffer, ShouldFlush, NextAccepts>
where
    Buffer: Extend<Value> + Default,
    ShouldFlush: Fn(&Buffer, &Value) -> bool,
    NextAccepts: Accepts<Buffer>,
    for<'a> &'a Buffer: IntoIterator<Item = &'a Value>,
{
    fn accept(&self, value: Value) {
        let mut buffer = self.buffer.borrow_mut();
        buffer.extend(core::iter::once(value));

        let buffer_ref = &*buffer;
        let should_flush = buffer_ref
            .into_iter()
            .last()
            .map(|latest_value| (self.should_flush)(buffer_ref, latest_value))
            .unwrap_or(false);

        if should_flush {
            let batch = core::mem::take(&mut *buffer);
            drop(buffer);
            self.next_acceptor.accept(batch);
        }
    }
}
