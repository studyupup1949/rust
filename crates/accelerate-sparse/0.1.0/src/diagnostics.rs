//! Accelerate diagnostics are routed into the calling frame.
//!
//! Accelerate aborts on a parameter violation when no callback is installed. The callback is
//! reinstalled before every fallible entry point because the re-exported sys setter can replace or
//! remove it.
//!
//! The callback signature carries no context pointer, so the message reaches the caller through a
//! thread-local. The trampoline must not unwind across the FFI boundary. Messages from threads
//! Accelerate spawned internally are dropped, so diagnostic detail is best-effort and decisions
//! use the status.

use accelerate_sparse_sys as sys;
use core::ffi::{CStr, c_char};
use std::cell::RefCell;

thread_local! {
    static LAST_MESSAGE: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Records Accelerate's diagnostic for the current thread.
///
/// Every path returns normally because unwinding into C is undefined behaviour.
unsafe extern "C" fn report_error(message: *const c_char) {
    if message.is_null() {
        return;
    }
    // SAFETY: Accelerate passes a NUL-terminated string that stays valid for the duration of the
    // call, which is the only window in which it is read.
    let text = unsafe { CStr::from_ptr(message) }
        .to_string_lossy()
        .into_owned();

    // `try_with` rather than `with`: during thread teardown the slot is gone, and that must be a
    // dropped message rather than a panic.
    let _ = LAST_MESSAGE.try_with(|slot| {
        if let Ok(mut slot) = slot.try_borrow_mut() {
            *slot = Some(text);
        }
    });
}

/// Installs the callback before an entry point that can fail.
pub(crate) fn install() {
    // SAFETY: `report_error` is callable from any thread and cannot unwind.
    unsafe { sys::accsp_set_error_handler(Some(report_error)) };
}

/// Records `message` for tests of diagnostic-based error paths.
#[cfg(test)]
pub(crate) fn plant(message: &str) {
    let _ = LAST_MESSAGE.try_with(|slot| {
        if let Ok(mut slot) = slot.try_borrow_mut() {
            *slot = Some(message.to_owned());
        }
    });
}

/// Discards any message left by an earlier call.
pub(crate) fn clear() {
    let _ = LAST_MESSAGE.try_with(|slot| {
        if let Ok(mut slot) = slot.try_borrow_mut() {
            *slot = None;
        }
    });
}

/// Takes the message recorded on this thread, if any.
pub(crate) fn take() -> Option<String> {
    LAST_MESSAGE
        .try_with(|slot| slot.try_borrow_mut().ok().and_then(|mut slot| slot.take()))
        .ok()
        .flatten()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::ffi::CString;

    // Each `#[test]` runs on its own thread, so the thread-local starts empty; `clear` guards the
    // case regardless. This is the exact path `finish` drains a status's detail through.

    #[test]
    fn a_recorded_message_round_trips_through_take_then_the_slot_empties() {
        clear();
        assert_eq!(take(), None, "nothing has been recorded yet");

        let message = CString::new("parameter 3 is out of range").unwrap();
        // SAFETY: `message` is a live NUL-terminated string for the call, which is the only window
        // `report_error` reads it in.
        unsafe { report_error(message.as_ptr()) };

        assert_eq!(take().as_deref(), Some("parameter 3 is out of range"));
        assert_eq!(take(), None, "take drains the slot");
    }

    #[test]
    fn clear_discards_a_pending_message() {
        let message = CString::new("stale").unwrap();
        // SAFETY: as above.
        unsafe { report_error(message.as_ptr()) };
        clear();
        assert_eq!(take(), None);
    }

    #[test]
    fn a_null_message_is_ignored_rather_than_dereferenced() {
        clear();
        // SAFETY: `report_error` treats a null pointer as no message, by contract.
        unsafe { report_error(core::ptr::null()) };
        assert_eq!(take(), None);
    }
}
