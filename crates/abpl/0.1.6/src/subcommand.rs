use core::{
	fmt::Display,
	ops::{Deref, DerefMut},
};
use std::{
	ffi::OsStr,
	io::{self, IoSlice, Read, Write},
	process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio},
	sync::mpsc,
	thread::JoinHandle,
};

use crate::thread::ThreadPanicError;

#[cfg(feature = "app")]
use crate::providers::ProvidesExitCode;

#[cfg(feature = "app")]
use std::process::ExitCode;

#[derive(Debug, Clone, PartialEq, Eq, abpl::Error)]
#[cfg_attr(feature = "app", abpl_provider(ProvidesExitCode(unreachable!(), exit_code, ExitCode)))]
pub enum SubCommandErrorKind {
	#[cause(std::io::Error)]
	#[cfg_attr(feature = "app", abpl_provider(exit_code(cause)))]
	Spawn { cmd: &'static str },
	#[cause(std::io::Error)]
	#[cfg_attr(feature = "app", abpl_provider(exit_code(cause)))]
	Wait { cmd: &'static str },
	#[cause(std::io::Error)]
	#[cfg_attr(feature = "app", abpl_provider(exit_code(cause)))]
	Stdout { cmd: &'static str },
	#[cfg_attr(feature = "app", abpl_provider(exit_code(
		match status.code() {
			None => ExitCode::FAILURE,
			Some(code) => ExitCode::from(code as u8)
		}
	)))]
	NonZeroStatus { cmd: &'static str, status: ExitStatus },
}
impl Display for SubCommandErrorKind {
	fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
		match self {
			Self::Spawn { cmd } => f.write_fmt(format_args!("failed to spawn process {cmd:?}")),
			Self::Wait { cmd } => f.write_fmt(format_args!("failed to wait for process {cmd:?}")),
			Self::Stdout { cmd } => f.write_fmt(format_args!("failed to capture {cmd:?} output")),
			Self::NonZeroStatus { cmd, status } => match status.code() {
				Some(code) => f.write_fmt(format_args!("{cmd:?} ended with status code {code}")),
				None => f.write_fmt(format_args!("{cmd:?} ended unsuccessfully")),
			},
		}
	}
}

/// Runs the specified command with an inherited stdin, stdout, and stderr.
///
/// Returns `Ok(())` if the child process exited with status `0`.
pub fn cmd(cmd: &'static str, args: impl IntoIterator<Item = impl AsRef<OsStr>>) -> Result<(), SubCommandError> {
	let status = Command::new(cmd)
		.args(args)
		.spawn()
		.map_err_spawn(cmd)?
		.wait()
		.map_err_wait(cmd)?;
	if !status.success() {
		return Err(SubCommandError::non_zero_status(cmd, status));
	}
	Ok(())
}

/// Runs the specified command with an inherited stdout and stderr, but with a piped stdin.
pub fn cmd_stdin(
	cmd: &'static str,
	args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<CmdWithStdin, SubCommandError> {
	let mut child = Command::new(cmd)
		.args(args)
		.stdin(Stdio::piped())
		.spawn()
		.map_err_spawn(cmd)?;
	let stdin = child.stdin.take().expect("Stdio::piped()");
	Ok(CmdWithStdin { cmd, child, stdin })
}

/// Runs the specified command with an inherited stdin and stderr, but with a piped stdout.
pub fn cmd_stdout(
	cmd: &'static str,
	args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<Vec<u8>, SubCommandError> {
	let output = Command::new(cmd)
		.args(args)
		.stdin(Stdio::inherit())
		.stderr(Stdio::inherit())
		.output()
		.map_err_spawn(cmd)?;

	if !output.status.success() {
		return Err(SubCommandError::non_zero_status(cmd, output.status));
	}
	Ok(output.stdout)
}

/// Shove some data into this thing and call [Self::wait] when you're done.
pub struct CmdWithStdin {
	cmd: &'static str,
	child: Child,
	stdin: ChildStdin,
}
impl CmdWithStdin {
	/// Closes the stdin of the child process, and waits until it exits.
	///
	/// Will return `Err` if the process exited with a non-zero status.
	pub fn wait(mut self) -> Result<(), SubCommandError> {
		drop(self.stdin);
		let status = self.child.wait().map_err_wait(self.cmd)?;
		if !status.success() {
			return Err(SubCommandError::non_zero_status(self.cmd, status));
		}
		Ok(())
	}
}
impl Deref for CmdWithStdin {
	type Target = ChildStdin;
	fn deref(&self) -> &Self::Target {
		&self.stdin
	}
}
impl DerefMut for CmdWithStdin {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.stdin
	}
}
impl Write for CmdWithStdin {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.stdin.write(buf)
	}
	fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
		self.stdin.write_vectored(bufs)
	}
	fn flush(&mut self) -> io::Result<()> {
		self.stdin.flush()
	}
}
impl Write for &CmdWithStdin {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		(&self.stdin).write(buf)
	}
	fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
		(&self.stdin).write_vectored(bufs)
	}
	fn flush(&mut self) -> io::Result<()> {
		(&self.stdin).flush()
	}
}

/// Runs the specified command with an inherited stderr, but with a piped stdin and stdout.
pub fn cmd_transform(
	cmd: &'static str,
	args: impl IntoIterator<Item = impl AsRef<OsStr>>,
) -> Result<CmdWithTransform, SubCommandError> {
	// I originally was going to flush the stdout during CmdWithTransform's writes, but that may cause a deadlock
	// situation where the child's stdout is fully backpressured and won't clear because stdin is also fully
	// backpressured and is currently being written to. So it seems like the most robust thing to do is handle the
	// input and output in parallel.
	//
	// Anyway, let's start the thread before attempting to spawn the process. If we did this after spawning the child
	// process but the stdout consumer thread fails to spawn for whatever reason, the child process would be abandoned
	// and lead to resource-leaking corner-cases.
	let (thread_tx, thread_rx) = mpsc::sync_channel::<ChildStdout>(0);

	let stdout_handle = std::thread::Builder::new()
		.name("subcmd stdout".into())
		.spawn(move || -> io::Result<Vec<u8>> {
			let Ok(mut stdout) = thread_rx.recv() else {
				// The handle has been abandoned, probably because the process failed to spawn.
				// Nothing to do but exit cleanly.
				return Ok(Vec::new());
			};
			let mut result = Vec::new();
			stdout.read_to_end(&mut result)?;
			Ok(result)
		})
		// Somewhat inaccurate, but afaik this is only likely to happen when resources are exhausted
		.map_err_spawn(cmd)?;

	let mut child = Command::new(cmd)
		.args(args)
		.stdin(Stdio::piped())
		.stderr(Stdio::inherit())
		.stdout(Stdio::piped())
		.spawn()
		.map_err_spawn(cmd)?;
	let stdin = child
		.stdin
		.take()
		.expect("stdin should have been made available using Stdio::piped()");

	thread_tx
		.send(
			child
				.stdout
				.take()
				.expect("stdout should have been made available using Stdio::piped()"),
		)
		.expect("the thread that would consume this value was already confirmed to have successfully spawned");

	Ok(CmdWithTransform {
		cmd,
		child,
		stdin,
		stdout_handle,
	})
}

/// Shove some data into this thing and call [Self::wait] or [Self::wait_ignoring_status] when you're done.
pub struct CmdWithTransform {
	cmd: &'static str,
	child: Child,
	stdin: ChildStdin,
	stdout_handle: JoinHandle<io::Result<Vec<u8>>>,
}
impl CmdWithTransform {
	/// Ends the stdin of the child process, and waits until it exits, returning the buffered contents of
	/// stdout.
	///
	/// Will return `Err` if the process exited with a non-zero status.
	pub fn wait(self) -> Result<Vec<u8>, SubCommandError> {
		let cmd = self.cmd;
		let (status, stdout) = self.wait_ignoring_status()?;
		if !status.success() {
			return Err(SubCommandError::non_zero_status(cmd, status));
		}
		Ok(stdout)
	}

	/// Ends the stdin of the child process, and waits until it exits, returning the buffered contents of
	/// stdout.
	///
	/// Will _not_ return `Err` if the process exited with a non-zero status.
	pub fn wait_ignoring_status(mut self) -> Result<(ExitStatus, Vec<u8>), SubCommandError> {
		drop(self.stdin);
		let status = self.child.wait().map_err_wait(self.cmd)?;
		let stdout = self
			.stdout_handle
			.join()
			.map_err(|err| io::Error::other(ThreadPanicError::from(err)))
			.map_err_stdout(self.cmd)?
			.map_err_stdout(self.cmd)?;
		Ok((status, stdout))
	}
}
impl Deref for CmdWithTransform {
	type Target = ChildStdin;
	fn deref(&self) -> &Self::Target {
		&self.stdin
	}
}
impl DerefMut for CmdWithTransform {
	fn deref_mut(&mut self) -> &mut Self::Target {
		&mut self.stdin
	}
}
impl Write for CmdWithTransform {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		self.stdin.write(buf)
	}
	fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
		self.stdin.write_vectored(bufs)
	}
	fn flush(&mut self) -> io::Result<()> {
		self.stdin.flush()
	}
}
impl Write for &CmdWithTransform {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		(&self.stdin).write(buf)
	}
	fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
		(&self.stdin).write_vectored(bufs)
	}
	fn flush(&mut self) -> io::Result<()> {
		(&self.stdin).flush()
	}
}
