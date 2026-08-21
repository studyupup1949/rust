#![allow(dead_code)]
extern crate http;

use std::time::Duration; 
use http::{Request, Response};

pub struct AdaClient {
    pub ada_io_username: String,
    pub ada_io_key: String,
    pub url: String,
}

impl AdaClient {
    pub fn set(&mut self, n1:String, n2:String){
        self.ada_io_username = n1;
        self.ada_io_key = n2;
    }


    pub fn post(&mut self, n3:String, data:String){
        let ada_io_feedkey:String = n3;
        let url:String = format!("https://io.adafruit.com/api/v2/{:}/feeds/{:}/data", 
        self.ada_io_username, ada_io_feedkey);
        Request::builder()
            .uri(url)
            .header("X-AIO-Key: ", &(self.ada_io_key))
            .body(data)
            .unwrap();
    }
   
}