use std::time::Instant;
use std::f32;

pub struct Time {
	time: Instant,
}

#[cfg(any(target_os = "linux", target_os = "macos", target_os = "android"))]
mod sleep {
	#[repr(C)]
	struct TimeSpec {
		seconds: isize,
		nanoseconds: isize,
	}

	extern {
		fn nanosleep(a: *const TimeSpec, b: usize) -> i32;
	}

	pub fn sleep(seconds: f32) {
		let secs = seconds as isize;
		let nanos = (seconds - secs as f32) * 1_000_000_000.0;
		let timespec = TimeSpec {
			seconds: secs,
			nanoseconds: nanos as isize,
		};

		unsafe {
			nanosleep(&timespec, 0);
		}
	}
}

#[cfg(target_os = "windows")]
mod sleep {
	#[link(name = "winmm")]
	#[link(name = "gdi32")]
	extern "system" {
		pub fn Sleep(d_word: u32) -> ();
		pub fn timeBeginPeriod(u_period: u32) -> u32;
	}

	pub fn sleep(seconds: f32) {
		let secs = seconds as u64;
		let millis = (seconds - secs as f32) * 1_000.0;

		unsafe {
			timeBeginPeriod(1); // necessary for accuracy
			Sleep(millis as u32);
		}
	}
}

impl Time {
	pub fn now() -> Time {
		Time { time: Instant::now() }
	}

	pub fn seconds_since(&self) -> f32 {
		let duration = self.time.elapsed();
		let nanos : f32 = duration.subsec_nanos() as f32
			/ 1_000_000_000.0;
		let secs : f32 = duration.as_secs() as f32;
		return secs + nanos;
	}

	pub fn half_linear_pulse(&self, rate_spr: f32) -> f32 {
		let passed = self.seconds_since();
		(passed % rate_spr) / rate_spr
	}

	pub fn full_linear_pulse(&self, rate_spr: f32) -> f32 {
		let passed = self.seconds_since();
		let rtn = (passed % rate_spr) / (rate_spr / 2.0);
		if rtn > 1.0 {
			2.0 - rtn
		}else{
			rtn
		}
	}

	pub fn full_smooth_pulse(&self, rate_spr: f32) -> f32 {
		1.0 - (((self.full_linear_pulse(rate_spr) * f32::consts::PI)
			.cos() + 1.0) / 2.0)
	}

	pub fn half_smooth_pulse(&self, rate_spr: f32) -> f32 {
		1.0 - (((self.half_linear_pulse(rate_spr) * f32::consts::PI)
			.cos() + 1.0) / 2.0)
	}

	pub fn sleep(seconds: f32) -> f32 {
		let time = Time::now();
		sleep::sleep(seconds);
		time.seconds_since()
	}
}
