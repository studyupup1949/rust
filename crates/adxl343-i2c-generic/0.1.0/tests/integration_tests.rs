#![allow(unused)]
#![cfg(all(
    target_arch = "aarch64",
    target_os = "linux",
    target_env = "gnu"
))]
mod helper;
use linux_embedded_hal::{I2cdev, I2CError};
use adxl343::{
	adxl343_interface::{ADXL343Interface, ADXL343Error},
	registers::accel_configs::{OutputDataRate, Alignment},
	utils::settings::ADXL343Settings
};
										
type SensorInterface = ADXL343Interface<I2cdev>;
type SensorError = ADXL343Error<I2CError>;

//ensures that we are speaking to the device with id 0x53 (ie the adxl343)
#[test]
fn confim_device() -> Result<(), SensorError>
{
	let mut sensor  = setup_i2c_interface_with_adxl343()?;
   	sensor.confirm_device()?;
    	Ok(())
}

//confirms that regardless of left or right justification, the resulting measurement (in lsb's) is equal
#[test]
fn left_right_justification_invariance() -> Result<(), SensorError>
{
	let mut sensor = setup_i2c_interface_with_adxl343()?;
	let mut sensor_right_justified = create_sensor_with_justification(
		sensor,
		Alignment::right
	)?;	

	let z_axis_reading_r = read_axis(&mut sensor_right_justified, Axis::Z)?;
	
	let mut sensor_left_justified = create_sensor_with_justification(
		sensor_right_justified, //cannot use sensor, since it was prev moved (8 lines back) 
		Alignment::left
	)?;

	let z_axis_reading_l = read_axis(&mut sensor_left_justified, Axis::Z)?;

	assert_eq!(z_axis_reading_r, z_axis_reading_l);
	Ok(())	
}

//Tests the maximum sampling rate which can be achieved through the i2c-dev system (Raspian-4B)
#[test]
fn sampling_rate_limit() -> Result<(), SensorError>
{
	use std::time::Instant;
	let instant = Instant::now();
	let mut delta_t_between_measurements: [u128; 3] =  [0x0; 3];	

	let mut sensor = setup_i2c_interface_with_adxl343()?;
	let mut sensor_fast_odr = create_sensor_with_sample_rate(
		sensor,
		OutputDataRate::Hz3200
	)?;

	let (since_t_0, t_0) = measure_time_between_consecutive_measurements(
		&mut sensor_fast_odr
	)?;

	let delta_t_vec: Vec<u128> = compute_delta_t_between_measurements(since_t_0, t_0);

	let bw_rate_val = sensor_fast_odr.read_register(0x2c)?;
	println!("{:b}", bw_rate_val);

	println!("{:?}, in milliseconds", delta_t_vec );	
	Ok(())
}

//samples time since t_0, immediately after sample is taken
fn measure_time_between_consecutive_measurements(sensor: &mut SensorInterface) ->
Result<(Vec<u128>, u128), SensorError>
{
	use std::time::{Duration, Instant};
	let t_0_instant = Instant::now(); 
	let t_0: Duration = t_0_instant.elapsed();	

	let mut instants = Vec::<Duration>::new();
	read_axis(sensor, Axis::Z)?; instants.push(t_0_instant.elapsed()); 
	read_axis(sensor, Axis::Z)?; instants.push(t_0_instant.elapsed());
	read_axis(sensor, Axis::Z)?; instants.push(t_0_instant.elapsed());	
	read_axis(sensor, Axis::Z)?; instants.push(t_0_instant.elapsed());

	let since_t_0: Vec<u128> = instants.iter_mut().map(
		|duration: &mut Duration| duration.as_micros() 
	).collect();

	Ok((since_t_0, t_0.as_micros()))
}


//samples[i] = samples[i] - samples[i-1], samples[i] stores time since t_0
fn compute_delta_t_between_measurements(mut samples: Vec<u128>, t_0: u128) -> Vec<u128>{
	if samples.len() == 0 {return samples;}
	
	samples[0] -= t_0;
	for i in 1..samples.len(){
		samples[i] -= samples[i-1];
	};
	samples
}

//returns the sensor, but with specified odr and measurements turned on 
fn create_sensor_with_sample_rate(mut sensor: SensorInterface, odr: OutputDataRate )
-> Result<SensorInterface, SensorError>
{
	sensor.turn_off_measurements()?;
	let (i2c, mut settings) = sensor.destroy();
	settings.set_odr(odr);
	let mut sensor = ADXL343Interface::<I2cdev>::new(i2c);

	sensor.with_settings(settings)?;
	sensor.init()?; sensor.begin_measurements()?;
	let _ = sensor.read_full_sample()?; //flush data buffer within sensor hardware
	Ok(sensor)	
}

//returns the sensor, but with specified justification and measurements turned on 
fn create_sensor_with_justification(mut sensor: SensorInterface, justification: Alignment)
-> Result<SensorInterface, SensorError>
{
	sensor.turn_off_measurements()?;
	let (i2c, mut settings) = sensor.destroy();
	settings.set_justification(justification);
	let mut sensor = ADXL343Interface::<I2cdev>::new(i2c);

	sensor.with_settings(settings)?;
	sensor.init()?; sensor.begin_measurements()?;
	let _ = sensor.read_full_sample()?; //flush data buffer within sensor hardware
	Ok(sensor)	
}  


//reads the specified axis (in g's) corresponding to the ADXL343 sensor
#[inline]
fn read_axis(sensor: &mut SensorInterface, axis: Axis)
-> Result<f32, SensorError> 
{
	let reading: [u8; 6] = sensor.read_full_sample()?;
	let axis_reading = match axis {
		Axis::X => [reading[0], reading[1]],
		Axis::Y => [reading[2], reading[3]],
		Axis::Z => [reading[4], reading[5]],
	}; 
	let axis_reading: i16 = sensor.axis_value_raw(axis_reading);
	Ok(sensor.axis_value(axis_reading))
}

pub fn setup_i2c_interface_with_adxl343() 
-> Result<ADXL343Interface<I2cdev>, I2CError>
{
    	//initializes an i2c device through the embedded linux hal lib
    	let i2c = I2cdev::new("/dev/i2c-1")?;
	let mut sensor = ADXL343Interface::new(i2c);
	Ok(sensor)
}


//helper enum to specify which axis to read
enum Axis{
	X,
	Y,
	Z
}



