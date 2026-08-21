use abbegm::tokio_peer::EgmPeer;
use nalgebra::Vector3;
use std::convert::TryInto;
use std::path::PathBuf;
use std::time::Duration;
use std::time::Instant;
use structopt::StructOpt;
use structopt::clap::AppSettings;
use tokio::time::timeout;

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// Move the robot back and forth in a straigh line using cartesian speed messages.
#[derive(Debug, StructOpt)]
#[structopt(setting = AppSettings::ColoredHelp)]
#[structopt(setting = AppSettings::DeriveDisplayOrder)]
#[structopt(setting = AppSettings::UnifiedHelpMessage)]
struct Options {
	/// Local address to bind to.
	#[structopt(long)]
	#[structopt(value_name = "HOST:PORT")]
	#[structopt(default_value = "[::]:6510")]
	bind: String,

	/// The speed in [mm/s] at which to move.
	#[structopt(long)]
	#[structopt(default_value = "100")]
	speed: f64,

	/// Start the ramp at this speed.
	#[structopt(long)]
	ramp_speed: f64,

	/// Time in [s] over which to ramp up/down speed.
	#[structopt(long)]
	#[structopt(default_value = "0")]
	ramp_time: f64,

	/// Maximum noise in [mm/s] to add add to speed.
	#[structopt(long)]
	#[structopt(default_value = "0")]
	speed_noise: f64,

	/// The maximum distance in [mm] to travel before reversing direction.
	#[structopt(long, short)]
	#[structopt(default_value = "400")]
	distance: f64,

	/// The X component of the direction vector.
	#[structopt(long, short = "x")]
	direction_x: f64,

	/// The Y component of the direction vector.
	#[structopt(long, short = "y")]
	direction_y: f64,

	/// The Z component of the direction vector.
	#[structopt(long, short = "z")]
	direction_z: f64,

	/// Confirm that the robot should perform motion.
	#[structopt(long)]
	confirm_motion: bool,

	/// Save the received state and send setpoints to a CSV file.
	#[structopt(long)]
	#[structopt(value_name = "LOG.csv")]
	log: Option<PathBuf>,
}

async fn do_main(options: Options) -> Result<(), ()> {
	if !options.confirm_motion {
		return Err(log::error!("refusing to send motion commands to the robot without --confirm-motion flag"))
	}

	let mut log_file = match &options.log {
		None => None,
		Some(path) => {
			let file = std::fs::File::create(path)
				.map_err(|e| log::error!("failed to create log file {}: {}", path.display(), e))?;
			Some(std::io::BufWriter::new(file))
		}
	};

	if let Some(log_file) = log_file.as_mut() {
		log_header(log_file).map_err(|e| log::error!("failed to write header to CSV log: {}", e))?;
	}

	let mut peer = EgmPeer::bind(&options.bind).await
		.map_err(|e| log::error!("failed to bind to local enpoint {}: {}", options.bind, e))?;

	let local_address = peer.socket().local_addr()
		.map_err(|e| log::error!("failed to get local socket address: {}", e))?;

	log::info!("Listening for messages on {}", local_address);

	let (state, _address) = peer.recv_from().await
		.map_err(|e| log::error!("failed to receive robot state: {}", e))?;

	log::info!("Received initial robot state.");

	// Retrieve start pose and compute center of circle.
	let start_time = Instant::now();
	let start_pose : nalgebra::Isometry3<f64> = state
		.feedback_pose()
		.ok_or_else(|| log::error!("missing `feedback.pose` in robot_message"))?
		.try_into()
		.map_err(|e| log::error!("failed to convert pose to isometry: {}", e))?;

	let start_position = start_pose.translation.vector;
	let direction = Vector3::new(options.direction_x, options.direction_y, options.direction_z).normalize();

	let mut sequence_number = 0u32;
	let mut invert_direction = false;
	let mut invert_time = start_time;

	// Install signal handlers to make sure we exit cleanly, and the log isn't truncated.
	let stop = Arc::new(AtomicBool::new(false));
	{
		use tokio::signal::unix::{signal, SignalKind};
		let mut sigterm = signal(SignalKind::terminate()).map_err(|e| log::error!("failed to install SIGTERM handler: {}", e))?;
		let mut sigint = signal(SignalKind::interrupt()).map_err(|e| log::error!("failed to install SIGINT handler: {}", e))?;
		let stop = stop.clone();
		tokio::spawn(async move {
			let signal = tokio::select!(
				_ = sigterm.recv() => "SIGTERM",
				_ = sigint.recv() => "SIGINT",
			);
			log::info!("caught {}, exitting", signal);
			stop.store(true, Ordering::Relaxed);
		});
	}

	use rand::distributions::Distribution;
	let noise_distribution = rand::distributions::Uniform::new_inclusive(-options.speed_noise, options.speed_noise);

	while !stop.load(Ordering::Relaxed) {
		let (state, address) = match tokio::time::timeout(Duration::from_millis(50), peer.recv_from()).await {
			Ok(Ok(x)) => x,
			Ok(Err(e)) => return Err(log::error!("failed to receive robot state: {}", e)),
			Err(tokio::time::Elapsed { .. }) => continue,
		};

		let time = state.feedback_time().ok_or_else(|| log::error!("missing `feedback.clock` in robot message"))?;
		log::debug!("Received robot state message from {}:", address);

		let current_pose : nalgebra::Isometry3<f64> = state
			.feedback_pose()
			.ok_or_else(|| log::error!("missing `feedback.pose` in robot_message"))?
			.try_into()
			.map_err(|e| log::error!("failed to convert pose to isometry: {}", e))?;
		let current_position = current_pose.translation.vector;

		// Project the position difference on the direction vector to get the travelled distance in the configured direction.
		let current_distance = (current_position - start_position).dot(&direction);

		// If we're past the target distance or starting point, update the direction.
		if current_distance >= options.distance && !invert_direction {
			invert_time = Instant::now();
			invert_direction = true;
		} else if current_distance <= 0.0 && invert_direction {
			invert_time = Instant::now();
			invert_direction = false;
		}

		let direction_factor = if invert_direction {
			-1.0
		} else {
			1.0
		};

		let ramp_factor;
		if options.ramp_time != 0.0 {
			ramp_factor = (invert_time.elapsed().as_secs_f64() / options.ramp_time).min(1.0);
		} else {
			ramp_factor = 1.0
		}

		let target_speed = direction_factor * (ramp_factor * options.ramp_speed + options.speed) + noise_distribution.sample(&mut rand::rngs::OsRng);
		let target_speed = direction * target_speed;

		if let Some(log_file) = log_file.as_mut() {
			log_step(log_file, start_time.elapsed(), &current_pose, &target_speed)
				.map_err(|e| log::error!("failed to write header to CSV log: {}", e))?
		}

		let message = abbegm::msg::EgmSensor::pose_target_with_speed(sequence_number, current_pose, target_speed, time);
		peer.send_to(&message, &address)
			.await
			.map_err(|e| log::error!("failed to send message to robot: {}", e))?;
		sequence_number = sequence_number.wrapping_add(1);
	}

	Ok(())
}

fn log_header<W: std::io::Write>(out: &mut W) -> std::io::Result<()> {
	writeln!(out, "time,feedback_position_x,feedback_position_y,feedback_position_z,setpoint_speed_z,setpoint_speed_y,setpoint_speed_z")
}

fn log_step<W: std::io::Write>(out: &mut W, time: std::time::Duration, feedback_pose: &nalgebra::Isometry3<f64>, target_speed: &Vector3<f64>) -> std::io::Result<()> {
	writeln!(out, "{:.},{:.},{:.},{:.},{:.},{:.},{:.}",
		time.as_secs_f64(),
		feedback_pose.translation.x,
		feedback_pose.translation.y,
		feedback_pose.translation.z,
		target_speed.x,
		target_speed.y,
		target_speed.z,
	)
}

#[tokio::main]
async fn main() {
	env_logger::from_env(env_logger::Env::default().default_filter_or("info")).init();

	if let Err(()) = do_main(Options::from_args()).await {
		std::process::exit(1);
	}
}
