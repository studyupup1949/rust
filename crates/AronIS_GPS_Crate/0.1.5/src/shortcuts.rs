use std::thread;
use std::time::Duration;

pub fn sleep(millis:u64){
    thread::sleep(Duration::from_millis(millis));
}