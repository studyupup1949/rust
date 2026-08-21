//! Bindings to a XNG MonoCore implementation of the ARINC653 P1/P2/P4 API

#![no_std]
#![deny(missing_docs)]
#![deny(rustdoc::broken_intra_doc_links)]

/// This module contains bindings to the services defined by ARINC653
pub mod bindings {
    #![allow(clippy::redundant_static_lifetimes)]
    #![allow(dead_code)]
    #![allow(missing_docs)]
    #![allow(non_camel_case_types)]
    #![allow(non_snake_case)]
    #![allow(non_upper_case_globals)]
    include!("bindings.rs");
}

/// This module contains the mappings from [crate::bindings] to [a653rs::bindings]
pub mod apex;

/// This panic handler reports the reason for a panic to the hypervisor and
/// then enters an infinite loop
#[cfg(feature = "panic_handler")]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    log::error!("{info}");

    let message = match info.payload().downcast_ref::<&str>() {
        Some(str) => str,
        None => "panic of unknown cause occured",
    };

    let last_index = core::cmp::min(bindings::MAX_ERROR_MESSAGE_SIZE as usize, message.len());

    // This function can fail if the message len in bytes is longer than
    // MAX_ERROR_MESSAGE_SIZE. To make it safe to unwrap, we have to shorten
    // the message if it exceeds the allowed length. As the slicing only happen
    // on the byte level, cutting of a multi-byte char (UTF-8) will not yield
    // internal panic, but of course it might disturb the hypervisors output.
    <apex::XngHypervisor as a653rs::prelude::ApexErrorP4Ext>::raise_application_error(
        &message.as_bytes()[0..last_index],
    )
    .unwrap();

    loop {}
}
