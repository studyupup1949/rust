//! Queuing port extension traits

#[cfg(feature = "alloc")]
use alloc::vec;

#[cfg(feature = "alloc")]
extern crate alloc;

use a653rs::prelude::*;
use postcard::de_flavors::Slice as DeSlice;
use postcard::ser_flavors::Slice as SerSlice;
use serde::{Deserialize, Serialize};

use crate::error::*;

/// Postcard extension trait for queuing port sender
pub trait QueuingPortSenderExt {
    /// Send a type using an a653rs [`QueuingPortSender`]
    ///
    /// # Example
    /// ```rust
    /// use a653rs_postcard::prelude::*;
    /// # use a653rs::prelude::*;
    /// # use std::str::FromStr;
    /// # use std::time::Duration;
    /// # use mock::MockHyp as Hypervisor;
    /// # #[path = "../tests/mock.rs"]
    /// # mod mock;
    /// # Hypervisor::run_test(|mut ctx| {
    /// # let port = ctx
    /// #     .create_queuing_port_sender(Name::from_str("").unwrap(), 500, 10, QueuingDiscipline::Fifo)
    /// #     .unwrap();
    ///
    /// let port: QueuingPortSender<Hypervisor> = port;
    /// port.send_type(String::from("Typed Data"), SystemTime::Infinite).unwrap();
    /// # })
    /// ```
    #[cfg(feature = "alloc")]
    fn send_type<T>(&self, p: T, timeout: SystemTime) -> Result<(), SendError>
    where
        T: Serialize;

    /// Send a type using an a653rs [`QueuingPortSender`]
    ///
    /// Requires a buffer `buf` for serialization.
    ///
    /// # Example
    /// ```rust
    /// use a653rs_postcard::prelude::*;
    /// # use a653rs::prelude::*;
    /// # use std::str::FromStr;
    /// # use std::time::Duration;
    /// # use mock::MockHyp as Hypervisor;
    /// # #[path = "../tests/mock.rs"]
    /// # mod mock;
    /// # Hypervisor::run_test(|mut ctx| {
    /// # let port = ctx
    /// #     .create_queuing_port_sender(Name::from_str("").unwrap(), 500, 10, QueuingDiscipline::Fifo)
    /// #     .unwrap();
    ///
    /// let port: QueuingPortSender<Hypervisor> = port;
    /// let mut buf = [0; 500];
    /// port.send_type_buf(String::from("Typed Data"), SystemTime::Infinite, &mut buf).unwrap();
    /// # })
    /// ```
    fn send_type_buf<T>(&self, p: T, timeout: SystemTime, buf: &mut [u8]) -> Result<(), SendError>
    where
        T: Serialize;
}

/// Postcard extension trait for queuing ports receiver
pub trait QueuingPortReceiverExt {
    /// Receive a type using an a653rs [`QueuingPortReceiver`]
    ///
    /// # Example
    /// ```rust
    /// use a653rs_postcard::prelude::*;
    /// # use a653rs::prelude::*;
    /// # use std::str::FromStr;
    /// # use std::time::Duration;
    /// # use mock::MockHyp as Hypervisor;
    /// # #[path = "../tests/mock.rs"]
    /// # mod mock;
    /// # Hypervisor::run_test(|mut ctx| {
    /// # let src_port = ctx
    /// #     .create_queuing_port_sender(Name::from_str("").unwrap(), 500, 10, QueuingDiscipline::Fifo)
    /// #     .unwrap();
    /// # let port = ctx
    /// #     .create_queuing_port_receiver(Name::from_str("").unwrap(), 500, 10, QueuingDiscipline::Fifo)
    /// #     .unwrap();
    ///
    /// let port: QueuingPortReceiver<Hypervisor> = port;
    /// # src_port.send_type(String::default(), SystemTime::Infinite).unwrap();
    /// port.recv_type::<String>(SystemTime::Infinite).unwrap();
    /// # })
    /// ```
    #[cfg(feature = "alloc")]
    fn recv_type<T>(&self, timeout: SystemTime) -> Result<(T, QueueOverflow), QueuingRecvError>
    where
        T: for<'a> Deserialize<'a>;

    /// Receive a type using an a653rs [`QueuingPortReceiver`]
    ///
    /// Requires a buffer `buf` for receiving and deserializing the data.
    ///
    /// # Example
    /// ```rust
    /// use a653rs_postcard::prelude::*;
    /// # use a653rs::prelude::*;
    /// # use std::str::FromStr;
    /// # use std::time::Duration;
    /// # use mock::MockHyp as Hypervisor;
    /// # #[path = "../tests/mock.rs"]
    /// # mod mock;
    /// # Hypervisor::run_test(|mut ctx| {
    /// # let src_port = ctx
    /// #     .create_queuing_port_sender(Name::from_str("").unwrap(), 500, 10, QueuingDiscipline::Fifo)
    /// #     .unwrap();
    /// # let port = ctx
    /// #     .create_queuing_port_receiver(Name::from_str("").unwrap(), 500, 10, QueuingDiscipline::Fifo)
    /// #     .unwrap();
    ///
    /// let port: QueuingPortReceiver<Hypervisor> = port;
    /// let mut buf = [0; 500];
    /// # src_port.send_type_buf(String::default(), SystemTime::Infinite, &mut buf).unwrap();
    /// port.recv_type_buf::<String>(SystemTime::Infinite, &mut buf).unwrap();
    /// # })
    /// ```
    fn recv_type_buf<'a, T>(
        &'a self,
        timeout: SystemTime,
        buf: &'a mut [u8],
    ) -> Result<(T, QueueOverflow), QueuingRecvBufError<'a>>
    where
        T: for<'b> Deserialize<'b>;
}

impl<Q: ApexQueuingPortP4Ext> QueuingPortSenderExt for QueuingPortSender<Q> {
    #[cfg(feature = "alloc")]
    fn send_type<T>(&self, p: T, timeout: SystemTime) -> Result<(), SendError>
    where
        T: Serialize,
    {
        let msg = postcard::to_allocvec(&p)?;
        self.send(&msg, timeout).map_err(SendError::from)
    }

    fn send_type_buf<T>(&self, p: T, timeout: SystemTime, buf: &mut [u8]) -> Result<(), SendError>
    where
        T: Serialize,
    {
        let buf =
            postcard::serialize_with_flavor::<T, SerSlice, &mut [u8]>(&p, SerSlice::new(buf))?;
        self.send(buf, timeout).map_err(SendError::from)
    }
}

impl<Q: ApexQueuingPortP4Ext> QueuingPortReceiverExt for QueuingPortReceiver<Q> {
    #[cfg(feature = "alloc")]
    fn recv_type<T>(&self, timeout: SystemTime) -> Result<(T, QueueOverflow), QueuingRecvError>
    where
        T: for<'a> Deserialize<'a>,
    {
        let mut buf = vec![0; self.size()];
        let (msg, overflow) = self.receive(&mut buf, timeout)?;
        match postcard::from_bytes(msg) {
            Ok(t) => Ok((t, overflow)),
            Err(e) => {
                let msg_len = msg.len();
                buf.truncate(msg_len);
                Err(QueuingRecvError::Postcard(e, buf))
            }
        }
    }

    fn recv_type_buf<'a, T>(
        &self,
        timeout: SystemTime,
        buf: &'a mut [u8],
    ) -> Result<(T, QueueOverflow), QueuingRecvBufError<'a>>
    where
        T: for<'b> Deserialize<'b>,
    {
        let (msg, overflow) = self.receive(buf, timeout)?;
        let msg_slice = DeSlice::new(msg);
        let mut deserializer = postcard::Deserializer::from_flavor(msg_slice);
        match T::deserialize(&mut deserializer) {
            Ok(t) => Ok((t, overflow)),
            Err(e) => Err(QueuingRecvBufError::Postcard(e, msg)),
        }
    }
}

#[cfg(test)]
#[path = "../tests"]
mod tests {
    use core::str::FromStr;
    use std::string::String;

    use a653rs::bindings::QueuingDiscipline;
    use a653rs::prelude::{Name, SystemTime};
    use mock::MockHyp;

    use crate::prelude::{QueuingPortReceiverExt, QueuingPortSenderExt};

    extern crate std;

    #[allow(clippy::duplicate_mod)]
    mod mock;

    #[test]
    fn queuing_type_buf() {
        MockHyp::run_test(|mut ctx| {
            let src_port = ctx
                .create_queuing_port_sender(
                    Name::from_str("").unwrap(),
                    500,
                    0,
                    QueuingDiscipline::Fifo,
                )
                .unwrap();
            let dest_port = ctx
                .create_queuing_port_receiver(
                    Name::from_str("").unwrap(),
                    500,
                    0,
                    QueuingDiscipline::Fifo,
                )
                .unwrap();

            let msg = String::from("Test");
            let mut buf = [0; 500];

            src_port
                .send_type_buf(msg.clone(), SystemTime::Infinite, &mut buf)
                .unwrap();
            let (rec, _): (String, _) = dest_port
                .recv_type_buf(SystemTime::Infinite, &mut buf)
                .unwrap();

            assert_eq!(msg, rec)
        })
    }

    #[test]
    fn const_queuing_type_buf() {
        MockHyp::run_test(|mut ctx| {
            let src_port = ctx
                .create_const_queuing_port_sender::<500, 0>(
                    Name::from_str("").unwrap(),
                    QueuingDiscipline::Fifo,
                )
                .unwrap();
            let dest_port = ctx
                .create_const_queuing_port_receiver::<500, 0>(
                    Name::from_str("").unwrap(),
                    QueuingDiscipline::Fifo,
                )
                .unwrap();

            let msg = String::from("Test");
            let mut buf = [0; 500];

            src_port
                .send_type_buf(msg.clone(), SystemTime::Infinite, &mut buf)
                .unwrap();
            let (rec, _): (String, _) = dest_port
                .recv_type_buf(SystemTime::Infinite, &mut buf)
                .unwrap();

            assert_eq!(msg, rec)
        })
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn queuing_type() {
        MockHyp::run_test(|mut ctx| {
            let src_port = ctx
                .create_queuing_port_sender(
                    Name::from_str("").unwrap(),
                    500,
                    0,
                    QueuingDiscipline::Fifo,
                )
                .unwrap();
            let dest_port = ctx
                .create_queuing_port_receiver(
                    Name::from_str("").unwrap(),
                    500,
                    0,
                    QueuingDiscipline::Fifo,
                )
                .unwrap();

            let msg = String::from("Test");

            src_port
                .send_type(msg.clone(), SystemTime::Infinite)
                .unwrap();
            let (rec, _): (String, _) = dest_port.recv_type(SystemTime::Infinite).unwrap();

            assert_eq!(msg, rec)
        })
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn const_queuing_type() {
        MockHyp::run_test(|mut ctx| {
            let src_port = ctx
                .create_const_queuing_port_sender::<500, 0>(
                    Name::from_str("").unwrap(),
                    QueuingDiscipline::Fifo,
                )
                .unwrap();
            let dest_port = ctx
                .create_const_queuing_port_receiver::<500, 0>(
                    Name::from_str("").unwrap(),
                    QueuingDiscipline::Fifo,
                )
                .unwrap();

            let msg = String::from("Test");

            src_port
                .send_type(msg.clone(), SystemTime::Infinite)
                .unwrap();
            let (rec, _): (String, _) = dest_port.recv_type(SystemTime::Infinite).unwrap();

            assert_eq!(msg, rec)
        })
    }
}
