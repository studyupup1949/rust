#[cfg(windows)]
/// https://learn.microsoft.com/en-us/windows/win32/api/realtimeapiset/nf-realtimeapiset-queryunbiasedinterrupttime
unsafe fn _active_time() -> std::io::Result<std::time::Duration> {
    let mut ticks = std::mem::MaybeUninit::uninit();
    let success = winapi::um::realtimeapiset::QueryUnbiasedInterruptTime(ticks.as_mut_ptr()) != 0;
    if success {
        Ok(std::time::Duration::from_nanos(ticks.assume_init() * 100))
    } else {
        Err(std::io::Error::last_os_error())
    }
}

/// Returns the active time of the OS, not including sleep/hibernation.
pub fn active_time() -> std::io::Result<std::time::Duration> {
    unsafe {
        _active_time()
    }
}