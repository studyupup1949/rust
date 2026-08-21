/// [a653rs::bindings::ApexErrorP1] and [a653rs::bindings::ApexErrorP4] bindings
pub mod error;
/// [a653rs::bindings::ApexPartitionP4] bindings
pub mod partition;
/// [a653rs::bindings::ApexProcessP1] and [a653rs::bindings::ApexProcessP4] bindings
pub mod process;
/// [a653rs::bindings::ApexQueuingPortP1] and [a653rs::bindings::ApexQueuingPortP4] bindings
pub mod queuing;
/// [a653rs::bindings::ApexSamplingPortP1] and [a653rs::bindings::ApexSamplingPortP4] bindings
pub mod sampling;
/// [a653rs::bindings::ApexScheduleP2] bindings
pub mod schedules;
/// [a653rs::bindings::ApexTimeP1] and [a653rs::bindings::ApexTimeP4] bindings
pub mod time;

/// Static struct representing a Xng Hypervisor
#[derive(Debug, Clone)]
pub struct XngHypervisor;
