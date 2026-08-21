
//! 
//!  ## Examples
//! ```
//! use AronIS_GPS_Crate::GPS;
//!
//! use rpi_embedded::uart::{Uart,Parity};
//! fn main(){
//! let mut gps = gps::GPS::new();
//!
//! loop{
//! gps.decoder();
//! }
//! }
//! ```
//! ## Important info:
//! ```
//! The output from the GPS is printed to the terminal and can additionally be read by reding the values from the struct
//! fx. in the example: pgs.altitude;
//! ```
mod gps;
fn main() {

let mut gps = gps::GPS::new();
loop{
gps.decoder();
}
}

// #[cfg(test)]
// mod test{
//     use super::*;
//     #[test]
//     fn test_standard(){
//         let mut gps_test1 = GPS::new();
//         let mut my_str = String::from("$GPGGA,112503.101,6507.4485,N,02255.5355,W,1,04,2.26,17.7,M,60.6,M,,*44");
//         gps_test1.decoder(my_str);
//         assert_eq!(gps_test1.standard,"$GPGGA")
//     }
//     #[test]
//     fn test_altitude(){
//         let mut gps_test2 = GPS::new();

//         let mut my_str = String::from("$GPGGA,112503.101,6507.4485,N,02255.5355,W,1,04,2.26,17.7,M,60.6,M,,*44");
//         gps_test2.decoder(my_str);
//         assert_eq!(gps_test2.altitude,"17.7")
//     }
//     #[test]
//     fn test_lattetude(){
//         let mut gps_test3 = GPS::new();
//         let mut my_str = String::from("$GPGGA,112503.101,6507.4485,N,02255.5355,W,1,04,2.26,17.7,M,60.6,M,,*44");
//         gps_test3.decoder(my_str);
//         assert_eq!(gps_test3.latetude,"6507.4485")
//     }

//     #[test]
//     fn test_longitude(){
//         let mut gps_test4 = GPS::new();
//         let mut my_str = String::from("$GPGGA,112503.101,6507.4485,N,02255.5355,W,1,04,2.26,17.7,M,60.6,M,,*44");
//         gps_test4.decoder(my_str);
//         assert_eq!(gps_test4.longitude,"02255.5355")
//     }
// }