

use rpi_embedded::uart::Uart;
use rpi_embedded::uart::Parity;
use std::time::Duration;
use std::thread;

#[derive(Debug)]
pub struct GPS{
    longitude:String,
    latetude:String,
    altitude:String,
    standard:String,
}
impl GPS{
    pub fn new()->Self{
        let mut _out = Self{
            longitude: "String".to_string(),
            latetude: "String".to_string(),
            altitude:"String".to_string(),
            standard:"$GPGGA".to_string(),
        };
        _out
    }
    // fn decoder(&mut self,data:String){
        pub fn decoder(&mut self){
        let mut uart = Uart::new(9600, Parity::None, 8, 1).unwrap();

        let mut uart = Uart::new(UART::baud_rate.get_val(), Parity::None, UART::parity_bit.get_val() as u8, UART::stop_bit.get_val() as u8).unwrap();
        uart.set_read_mode(1, Duration::default()).expect("uart set read");
        let gps_data = uart.read_until('\n').expect("read untill");
        // let gps_data = data; //Remove this when tests are done.
        let data_list = gps_data.split(',');
        let vec: Vec<&str> = data_list.collect();
        self.standard = vec[0].to_string();
        if vec[0].to_string() ==String::from("$GPGGA") {
            self.altitude = vec[9].to_string();
            self.latetude = vec[2].to_string();
            self.longitude = vec[4].to_string();
            // println!("{}",vec[0]);
            println!("Alt: {}{}",self.altitude,vec[10]);
            println!("Lat: {}",self.latetude);
            println!("Long: {}",self.longitude);
            thread::sleep(Duration::from_millis(1000));
        }
    }

}
pub enum UART{
    baud_rate,
    parity_bit,
    stop_bit,
}
impl UART{
    pub fn get_val(&self)->u32{
        let value:u32;
        match self{
            UART::baud_rate => {value = 9600}
           // UART::parody => {None}
            UART::parity_bit => {value = 8}
            UART::stop_bit => {value = 1}
        }
        value
    }
}

