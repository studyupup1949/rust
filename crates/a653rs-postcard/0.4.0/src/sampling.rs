//! Sampling port extension traits

#[cfg(feature = "alloc")]
use alloc::vec;

#[cfg(feature = "alloc")]
extern crate alloc;

use a653rs::prelude::*;
use postcard::de_flavors::Slice as DeSlice;
use postcard::ser_flavors::Slice as SerSlice;
use serde::{Deserialize, Serialize};

use crate::error::*;

/// Postcard extension trait for sampling port sources
pub trait SamplingPortSourceExt {
    // Send a type using an a653rs [`SamplingPortSource`]
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
    /// #     .create_sampling_port_source(Name::from_str("").unwrap(), 500)
    /// #     .unwrap();
    ///
    /// let port: SamplingPortSource<Hypervisor> = port;
    /// port.send_type(String::from("Typed Data")).unwrap();
    /// # })
    /// ```
    #[cfg(feature = "alloc")]
    fn send_type<T>(&self, p: T) -> Result<(), SendError>
    where
        T: Serialize;

    // Send a type using an a653rs [`SamplingPortSource`]
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
    /// #     .create_sampling_port_source(Name::from_str("").unwrap(), 500)
    /// #     .unwrap();
    ///
    /// let port: SamplingPortSource<Hypervisor> = port;
    /// let mut buf = [0; 500];
    /// port.send_type_buf(String::from("Typed Data"), &mut buf).unwrap();
    /// # })
    /// ```
    fn send_type_buf<T>(&self, p: T, buf: &mut [u8]) -> Result<(), SendError>
    where
        T: Serialize;
}

/// Postcard extension trait for sampling port destinations
pub trait SamplingPortDestinationExt {
    /// Receive a type using an a653rs [`SamplingPortDestination`]
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
    /// #     .create_sampling_port_source(Name::from_str("").unwrap(), 500)
    /// #     .unwrap();
    /// # let port = ctx
    /// #     .create_sampling_port_destination(Name::from_str("Port").unwrap(), 500, Duration::ZERO)
    /// #     .unwrap();
    ///
    /// let port: SamplingPortDestination<Hypervisor> = port;
    /// # src_port.send_type(String::default()).unwrap();
    /// let (validity, received_type) = port.recv_type::<String>().unwrap();
    /// # })
    /// ```
    #[cfg(feature = "alloc")]
    fn recv_type<T>(&self) -> Result<(Validity, T), SamplingRecvError>
    where
        T: for<'a> Deserialize<'a>;

    /// Receive a type using an a653rs [`SamplingPortDestination`]
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
    /// #     .create_sampling_port_source(Name::from_str("").unwrap(), 500)
    /// #     .unwrap();
    /// # let port = ctx
    /// #     .create_sampling_port_destination(Name::from_str("Port").unwrap(), 500, Duration::ZERO)
    /// #     .unwrap();
    ///
    /// let port: SamplingPortDestination<Hypervisor> = port;
    /// let mut buf = [0; 500];
    /// # src_port.send_type_buf(String::default(), &mut buf).unwrap();
    /// let (validity, received_type) = port.recv_type_buf::<String>(&mut buf).unwrap();
    /// # })
    /// ```
    fn recv_type_buf<'a, T>(
        &self,
        buf: &'a mut [u8],
    ) -> Result<(Validity, T), SamplingRecvBufError<'a>>
    where
        T: for<'b> Deserialize<'b>;
}

impl<Q: ApexSamplingPortP4Ext> SamplingPortSourceExt for SamplingPortSource<Q> {
    /// Receive a type using an a653rs [`SamplingPortDestination`]
    ///
    /// Requires a buffer `buf` for receiving and deserializing the data.
    #[cfg(feature = "alloc")]
    fn send_type<T>(&self, p: T) -> Result<(), SendError>
    where
        T: Serialize,
    {
        let msg = postcard::to_allocvec(&p)?;
        self.send(&msg).map_err(SendError::from)
    }

    fn send_type_buf<T>(&self, p: T, buf: &mut [u8]) -> Result<(), SendError>
    where
        T: Serialize,
    {
        let buf =
            postcard::serialize_with_flavor::<T, SerSlice, &mut [u8]>(&p, SerSlice::new(buf))?;
        self.send(buf).map_err(SendError::from)
    }
}

impl<Q: ApexSamplingPortP4Ext> SamplingPortDestinationExt for SamplingPortDestination<Q> {
    #[cfg(feature = "alloc")]
    fn recv_type<'a, T>(&self) -> Result<(Validity, T), SamplingRecvError>
    where
        T: for<'b> Deserialize<'b>,
    {
        let mut buf = vec![0; self.size() as usize];
        let (val, msg) = self.receive(&mut buf)?;
        match postcard::from_bytes(msg) {
            Ok(t) => Ok((val, t)),
            Err(e) => {
                let msg_len = msg.len();
                buf.truncate(msg_len);
                Err(SamplingRecvError::Postcard(e, val, buf))
            }
        }
    }

    fn recv_type_buf<'a, T>(
        &self,
        buf: &'a mut [u8],
    ) -> Result<(Validity, T), SamplingRecvBufError<'a>>
    where
        T: for<'b> Deserialize<'b>,
    {
        let (val, msg) = self.receive(buf)?;
        let msg_slice = DeSlice::new(msg);
        let mut deserializer = postcard::Deserializer::from_flavor(msg_slice);
        match T::deserialize(&mut deserializer) {
            Ok(t) => Ok((val, t)),
            Err(e) => Err(SamplingRecvBufError::Postcard(e, val, msg)),
        }
    }
}

#[cfg(test)]
#[path = "../tests"]
mod tests {
    use core::str::FromStr;
    use core::time::Duration;
    use std::string::String;

    use a653rs::prelude::Name;
    use mock::MockHyp;

    use crate::prelude::{SamplingPortDestinationExt, SamplingPortSourceExt};

    extern crate std;

    #[allow(clippy::duplicate_mod)]
    mod mock;

    #[test]
    fn sampling_type_buf() {
        MockHyp::run_test(|mut ctx| {
            let src_port = ctx
                .create_sampling_port_source(Name::from_str("").unwrap(), 500)
                .unwrap();
            let dest_port = ctx
                .create_sampling_port_destination(Name::from_str("").unwrap(), 500, Duration::ZERO)
                .unwrap();

            let msg = String::from("Test");
            let mut buf = [0; 500];

            src_port.send_type_buf(msg.clone(), &mut buf).unwrap();
            let (_, rec): (_, String) = dest_port.recv_type_buf(&mut buf).unwrap();

            assert_eq!(msg, rec)
        })
    }

    #[test]
    fn const_sampling_type_buf() {
        MockHyp::run_test(|mut ctx| {
            let src_port = ctx
                .create_const_sampling_port_source::<500>(Name::from_str("").unwrap())
                .unwrap();
            let dest_port = ctx
                .create_const_sampling_port_destination::<500>(
                    Name::from_str("").unwrap(),
                    Duration::ZERO,
                )
                .unwrap();

            let msg = String::from("Test");
            let mut buf = [0; 500];

            src_port.send_type_buf(msg.clone(), &mut buf).unwrap();
            let (_, rec): (_, String) = dest_port.recv_type_buf(&mut buf).unwrap();

            assert_eq!(msg, rec)
        })
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn sampling_type() {
        MockHyp::run_test(|mut ctx| {
            let src_port = ctx
                .create_sampling_port_source(Name::from_str("").unwrap(), 500)
                .unwrap();
            let dest_port = ctx
                .create_sampling_port_destination(Name::from_str("").unwrap(), 500, Duration::ZERO)
                .unwrap();

            let msg = String::from("Test");

            src_port.send_type(msg.clone()).unwrap();
            let (_, rec): (_, String) = dest_port.recv_type().unwrap();

            assert_eq!(msg, rec)
        })
    }

    #[cfg(feature = "alloc")]
    #[test]
    fn const_sampling_type() {
        MockHyp::run_test(|mut ctx| {
            let src_port = ctx
                .create_const_sampling_port_source::<500>(Name::from_str("").unwrap())
                .unwrap();
            let dest_port = ctx
                .create_const_sampling_port_destination::<500>(
                    Name::from_str("").unwrap(),
                    Duration::ZERO,
                )
                .unwrap();

            let msg = String::from("Test");

            src_port.send_type(msg.clone()).unwrap();
            let (_, rec): (_, String) = dest_port.recv_type().unwrap();

            assert_eq!(msg, rec)
        })
    }
}
