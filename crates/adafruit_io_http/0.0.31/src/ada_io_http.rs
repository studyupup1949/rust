#![allow(dead_code)]
extern crate http;

use std::time::Duration; 
use http::{Request, Response};

pub struct AdaClient {
    pub ada_io_username: String,
    pub ada_io_key: String,
}

impl AdaClient {
    pub fn set(n1:String, n2:String) -> Self {
        Self{
        ada_io_username:n1,
        ada_io_key:n2,
        }
    }

    pub fn post(&mut self, n3:String, data:String){
        let ada_io_feedkey:String = n3;
        let url:String = format!("api/v2/{:}/feeds/{:}//data", 
        self.ada_io_username, ada_io_feedkey);
        println!("Https request using uri: {:}",url);
        let head:String = format!("X-AIO-Key: {:}", self.ada_io_key);
        println!("Header: {:}", &head);
        Request::builder()
            .uri("https://io.adafruit.com/")
            .header(&head, &url)
            .body(&data)
            .unwrap();
    }
   
}