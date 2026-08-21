// SPDX-License-Identifier: Apache-2.0

//! Shared utility functions used across modules.

/// Check whether a process with the given PID is still alive.
pub fn is_process_alive(pid: i32) -> bool {
    std::process::Command::new("kill")
        .args(["-0", &pid.to_string()])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn current_process_is_alive() {
        #[allow(clippy::cast_possible_wrap)]
        let pid = std::process::id() as i32;
        assert!(is_process_alive(pid));
    }

    #[test]
    fn nonexistent_pid_is_not_alive() {
        assert!(!is_process_alive(99999));
    }
}
