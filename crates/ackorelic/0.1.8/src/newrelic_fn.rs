use crate::acko_segment::Segment;
use crate::nr_init::NR_APP;
use crate::transaction::Transaction;
use std::cell::RefCell;

use crate::nr_init::ENABLE_NEW_RELIC;
use newrelic_sys as ffi;
use std::ptr::null_mut;

//pub struct TLData {
//    transaction: Transaction,
//    segments: Vec<ffi::newrelic_segment_t>,
//}

thread_local! {
    pub static TL_TRANSACTION: RefCell<Option<Transaction>> = RefCell::new(
            match NR_APP.web_transaction("init") {
                Ok(trans) => Some(trans),
                Err(e) => {
                    println!("Error init web transaction:: {:?}", e);
                    None
                }
            }
        );

    //pub static TL_SEGMENT: RefCell<Segment> = RefCell::new(Segment::custom( TL_TRANSACTION.with(|tr| {*tr.borrow_mut()}), "init","Custom"));
}

pub fn nr_start_web_transaction(name: &str) -> () {
    //println!("Starting web transaction name : {}", name);
    if *ENABLE_NEW_RELIC {
        TL_TRANSACTION.with(|tr| {
            match NR_APP.web_transaction(name) {
                Ok(transaction) => *tr.borrow_mut() = Some(transaction),
                Err(e) => {
                    println!("Error in starting web transaction:: {:?}", e);
                }
            };
        });
    }
}

pub fn nr_end_transaction() {
    if *ENABLE_NEW_RELIC {
        TL_TRANSACTION.with(|tr| {
            tr.borrow_mut().as_mut().and_then(|trans| Some(trans.end()));
        });
    }
}

fn nullable_segment() -> Segment {
    let inner: *mut ffi::newrelic_segment_t = null_mut();
    Segment { inner }
}

pub fn nr_start_custom_segment(name: &str) -> Segment {
    //println!("Starting custom segment name : {}", name);
    if *ENABLE_NEW_RELIC {
        TL_TRANSACTION.with(|tr| {
            tr.borrow().as_ref().map_or_else(
                || nullable_segment(),
                |trans| Segment::start_custom_segment(trans, name),
            )
        })
    } else {
        nullable_segment()
    }
}

pub fn nr_end_custom_segment(segment: Segment) {
    //println!("Ending custom segment");
    if *ENABLE_NEW_RELIC {
        TL_TRANSACTION.with(|tr| {
            tr.borrow()
                .as_ref()
                .and_then(|trans| Some(segment.end_segment(trans)))
        });
    }
}
