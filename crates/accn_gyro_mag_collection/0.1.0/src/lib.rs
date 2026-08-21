#![no_std]
// use core::ops::Div;

// use nalgebra::{self, Point3, Rotation2, SimdRealField};

#[cfg(feature = "adxl345")]
pub mod adxl345;

// pub trait Accelrometer<T>
// where
//     T: nalgebra::Scalar + Copy + core::fmt::Debug + Div + SimdRealField,
// {
//     fn get_accel(&self) -> Point3<T>;
//     fn spirit_level_radians(&self) -> Rotation2<T> {
//         let accel = self.get_accel();
//         let x = accel.x;
//         let y = accel.y;
//         let z = accel.z;
//         let angle = (y / z).simd_atan2(x / z);
//         Rotation2::new(angle)
//     }
// }

// pub trait Gyroscope<T>
// where
//     T: nalgebra::Scalar + Copy + core::fmt::Debug + Div + SimdRealField + PartialOrd,
// {
//     /// Get the Gyroscope data as Angles per second in radiand. SI unit: rad/s
//     fn get_gyro(&self) -> Point3<T>;
//     fn is_stable(&self) -> bool {
//         let gyro = self.get_gyro();
//         let x = gyro.x;
//         let y = gyro.y;
//         let z = gyro.z;
//         let threshold = 0.1; // Adjust this threshold as needed
//         x.simd_abs() < threshold && y.simd_abs() < threshold && z.simd_abs() < threshold
//     }
// }

// pub trait Magnetometer {
//     fn get_mag(&self) -> Point3<f32>;
// }
