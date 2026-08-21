use std::ffi::CString;


use newrelic_sys as ffi;

use crate::{transaction::Transaction};

pub struct Segment {
    pub inner: *mut ffi::newrelic_segment_t,
}

impl Segment {
    pub fn start_custom_segment(transaction: &Transaction, name: &str) -> Self{
        let c_name = CString::new(name);
        let c_category = CString::new("Custom");
        let inner = unsafe {
                    ffi::newrelic_start_segment(
                        transaction.inner,
                        c_name.unwrap().as_ptr(),
                        c_category.unwrap().as_ptr(),
                    )
                };
        Segment { inner }
        /*let inner = match (c_name, c_category) {
            (Ok(c_name), Ok(c_category)) => {
                let inner = unsafe {
                    ffi::newrelic_start_segment(
                        transaction.inner,
                        c_name.as_ptr(),
                        c_category.as_ptr(),
                    )
                };
                if inner.is_null() {
                    error!(
                        "Could not create segment with name {} due to invalid transaction",
                        name
                    );
                    None
                } else {
                    Some(Segment { inner })
                }
            }
            _ => {
                error!(
                    "Could not create segment with name {}, category {}, due to NUL string in name or category",
                    name,
                    "Custom",
                );
                None
            }
        };
        debug!("Created segment");
        Segment { inner }*/
    }

    pub fn end_segment(mut self, transaction: &Transaction) {
        unsafe {
                ffi::newrelic_end_segment(transaction.inner, &mut self.inner);
            }
    }
}
//
//impl Drop for Segment{
//    fn drop(&mut self) {
//        self.e
//    }
//}