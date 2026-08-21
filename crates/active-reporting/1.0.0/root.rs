//^
//^ HEAD
//^

//> HEAD -> SUPER
use super::Report;


//^
//^ ROOT
//^

//> ROOT -> STRUCT
pub struct Root {
    chain: Vec<&'static str>
}

//> ROOT -> IMPLEMENTATION
impl Root {
    pub const fn new(name: &'static str) -> Self {return Self {
        chain: if name.is_empty() {Vec::new()} else {
            let mut chain = Vec::new();
            chain.push(name);
            chain
        }
    }}
    pub const fn chain(&self) -> &[&'static str] {return self.chain.as_slice()}
    pub const fn to<'valid, const NAME: &'static str>(&'valid mut self) -> Report<'valid, NAME> {
        if !NAME.is_empty() {self.chain.push(NAME);}
        return Report {
            chain: &mut self.chain
        }
    }
}

//> ROOT -> DEFAULT
const impl Default for Root {
    fn default() -> Self {return Self::new("Main")}
}

//> ROOT -> !SYNC
impl !Sync for Root {}