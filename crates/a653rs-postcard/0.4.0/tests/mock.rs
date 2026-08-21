use core::mem::MaybeUninit;
use std::sync::Mutex;
use std::vec::Vec;

use a653rs::bindings::{ApexQueuingPortP4, ApexSamplingPortP4, Validity};
use a653rs::prelude::StartContext;

extern crate std;

static mut SAMPLING_PORTS: Vec<u8> = Vec::new();
static mut QUEUING_PORTS: Vec<Vec<u8>> = Vec::new();
static SYNC: Mutex<()> = Mutex::new(());

pub struct MockHyp;

impl MockHyp {
    /// Prevents multiple tests from running concurrently
    /// Also clears all ports before starting with the next one
    pub fn run_test(t: fn(StartContext<MockHyp>)) {
        let ctx = unsafe { MaybeUninit::zeroed().assume_init() };
        let lock = SYNC.lock();
        unsafe {
            SAMPLING_PORTS.clear();
            QUEUING_PORTS.clear();
        }
        t(ctx);
        drop(lock);
    }
}

impl ApexSamplingPortP4 for MockHyp {
    fn create_sampling_port(
        _sampling_port_name: a653rs::bindings::SamplingPortName,
        _max_message_size: a653rs::prelude::MessageSize,
        _port_direction: a653rs::bindings::PortDirection,
        _refresh_period: a653rs::bindings::ApexSystemTime,
    ) -> Result<a653rs::prelude::SamplingPortId, a653rs::bindings::ErrorReturnCode> {
        Ok(0)
    }

    fn write_sampling_message(
        _sampling_port_id: a653rs::prelude::SamplingPortId,
        message: &[a653rs::prelude::ApexByte],
    ) -> Result<(), a653rs::bindings::ErrorReturnCode> {
        unsafe { SAMPLING_PORTS = message.to_vec() };
        Ok(())
    }

    unsafe fn read_sampling_message(
        _sampling_port_id: a653rs::prelude::SamplingPortId,
        out: &mut [a653rs::prelude::ApexByte],
    ) -> Result<
        (a653rs::prelude::Validity, a653rs::prelude::MessageSize),
        a653rs::bindings::ErrorReturnCode,
    > {
        let msg = unsafe { SAMPLING_PORTS.clone() };
        let len = out.len().min(msg.len());
        out[..len].copy_from_slice(&msg.as_slice()[..len]);

        Ok((Validity::Valid, len as u32))
    }
}

impl ApexQueuingPortP4 for MockHyp {
    fn create_queuing_port(
        _queuing_port_name: a653rs::bindings::QueuingPortName,
        _max_message_size: a653rs::prelude::MessageSize,
        _max_nb_message: a653rs::prelude::MessageRange,
        _port_direction: a653rs::bindings::PortDirection,
        _queuing_discipline: a653rs::prelude::QueuingDiscipline,
    ) -> Result<a653rs::prelude::QueuingPortId, a653rs::bindings::ErrorReturnCode> {
        Ok(0)
    }

    fn send_queuing_message(
        _queuing_port_id: a653rs::prelude::QueuingPortId,
        message: &[a653rs::prelude::ApexByte],
        _time_out: a653rs::bindings::ApexSystemTime,
    ) -> Result<(), a653rs::bindings::ErrorReturnCode> {
        unsafe { QUEUING_PORTS.push(message.to_vec()) };
        Ok(())
    }

    unsafe fn receive_queuing_message(
        _queuing_port_id: a653rs::prelude::QueuingPortId,
        _time_out: a653rs::bindings::ApexSystemTime,
        out: &mut [a653rs::prelude::ApexByte],
    ) -> Result<
        (a653rs::prelude::MessageSize, a653rs::prelude::QueueOverflow),
        a653rs::bindings::ErrorReturnCode,
    > {
        let msg = unsafe { QUEUING_PORTS.remove(0) };
        let len = out.len().min(msg.len());
        out[..len].copy_from_slice(&msg.as_slice()[..len]);

        Ok((len as u32, false))
    }

    fn get_queuing_port_status(
        _queuing_port_id: a653rs::prelude::QueuingPortId,
    ) -> Result<a653rs::prelude::QueuingPortStatus, a653rs::bindings::ErrorReturnCode> {
        unimplemented!()
    }

    fn clear_queuing_port(
        _queuing_port_id: a653rs::prelude::QueuingPortId,
    ) -> Result<(), a653rs::bindings::ErrorReturnCode> {
        unimplemented!()
    }
}
