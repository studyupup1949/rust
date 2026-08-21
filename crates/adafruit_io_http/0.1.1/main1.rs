
use std::time::Duration;
use std::thread;
extern crate ureq;
extern crate serde_json;


fn main() {
    let username = "Danny111";
    let feedkey = "modeon";


    loop {
        let r = ureq::get(&format!("https://io.adafruit.com/api/v2/{}/feeds/{}/data?limit=1", username, feedkey))
        .set("X-AIO-Key", "aio_PweL13M6yxh67mevUF79d4xcCEvx")
        .call();
        let json = r.into_json().unwrap();
        let data:String = json[0]["value"].as_str().unwrap().parse().unwrap();
        println!("{:?}", json);
        println!("{}", data);
        thread::sleep(Duration::from_secs(10));

        //client.post(("modeon").to_string(),("75").to_string());  
        
        ureq::post(&format!("https://io.adafruit.com/api/v2/{}/feeds/{}/data", username, feedkey))
        .set("X-AIO-Key", "aio_PweL13M6yxh67mevUF79d4xcCEvx")
        .send_form(&[("value", "75")]);

        
    }
}