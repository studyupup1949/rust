#![allow(dead_code)]

use std::string::String;
extern crate ureq;
extern crate serde_json;

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
        let ada_io_feedkey = n3;
        ureq::post(&format!("https://io.adafruit.com/api/v2/{}/feeds/{}/data", self.ada_io_username, ada_io_feedkey))
        .set("X-AIO-Key", &(format!("X-AIO-Key: {:}", self.ada_io_key)))
        .send_form(&[("value", &data)]);
    }

    pub fn get(&mut self, n3:String) -> String{
        let ada_io_feedkey = n3;
        let r = ureq::get(&format!("https://io.adafruit.com/api/v2/{}/feeds/{}/data?limit=1", self.ada_io_username, ada_io_feedkey))
        .set("X-AIO-Key", &(format!("X-AIO-Key: {:}", self.ada_io_key)))
        .call();
        let json = r.into_json().unwrap();
        let result:String = json[0]["value"].as_str().unwrap().parse().unwrap();
        result
    }   
   
}