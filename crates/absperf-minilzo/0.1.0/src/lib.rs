mod error;
pub use error::Error;
pub use error::Result;
pub use error::check;

use absperf_minilzo_sys::{__lzo_init_v2, lzo_version, lzo_callback_t, lzo_uint};
use std::os::raw::{c_int, c_short, c_long, c_char};
use std::mem::size_of;

pub fn init() -> Result<()> {
    let ret = unsafe {
        __lzo_init_v2(lzo_version(),
        size_of::<c_short>() as c_int,
        size_of::<c_int>() as c_int,
        size_of::<c_long>() as c_int,
        size_of::<u32>() as c_int, // lzo_uint32_t
        size_of::<lzo_uint>() as c_int,
        size_of::<usize>() as c_int, // lzo_sizeof_dict_t
        size_of::<*const c_char>() as c_int,
        size_of::<usize>() as c_int, // lzo_voidp
        size_of::<lzo_callback_t>() as c_int
        )
    };
    check(ret)
}

#[cfg(test)]
mod tests {
    use super::*;
    use absperf_minilzo_sys::{lzo_uint, lzo_callback_t};
    use std::os::raw::{c_int, c_short, c_long};

    fn size_of<T>() -> c_int {
        std::mem::size_of::<T>() as c_int
    }

    // Simple test to make sure a library function can be correctly called
    #[test]
    fn init_works() {
        assert_eq!(init(), Ok(()));
    }
}
