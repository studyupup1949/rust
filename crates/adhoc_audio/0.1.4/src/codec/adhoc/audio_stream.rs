use serde::{Deserialize, Serialize};

use super::*;

#[derive(Deserialize,Serialize)]
pub struct AudioStream {
    info: Option<StreamInfo>,
    stream: BitStream,
}

impl Default for AudioStream{
    fn default() -> Self {
        Self::new()
    }
}

impl AudioStream {
    pub fn new() -> Self {
        Self {
            info: None,
            stream: BitStream::new(),
        }
    }
    pub fn set_info(&mut self,info:Option<StreamInfo>){
        self.info = info;
    }
    pub fn info(&self) -> Option<StreamInfo> {
        self.info
    }
}

impl Deref for AudioStream {
    type Target = BitStream;
    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl DerefMut for AudioStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}
