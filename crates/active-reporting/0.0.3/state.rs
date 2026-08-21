//^
//^ STATE
//^

//> STATE -> BASE
pub const trait State {
    fn get<'valid>(&'valid self) -> &'valid Vec<&'static str>;
    fn get_mut<'valid>(&'valid mut self) -> &'valid mut Vec<&'static str>;
}

//> STATE -> DERIVED
pub const trait DerivedState<'valid>: const State + const Convert<'valid> {}

//> STATE -> CONVERT
pub const trait Convert<'valid> {
    fn convert<Current: const State>(value: &'valid mut Current) -> Self;
}


//^
//^ MAIN
//^

//> MAIN -> STRUCT
pub struct Main {
    pub chain: Vec<&'static str> = Vec::new()
}

//> MAIN -> STATE
const impl State for Main {
    fn get<'valid>(&'valid self) -> &'valid Vec<&'static str> {return &self.chain}
    fn get_mut<'valid>(&'valid mut self) -> &'valid mut Vec<&'static str> {return &mut self.chain}
}


//^
//^ SAME
//^

//> SAME -> STRUCT
pub struct Same<'this> {
    link: &'this mut Vec<&'static str>
} 

//> SAME -> STATE
const impl<'this> State for Same<'this> {
    fn get<'valid>(&'valid self) -> &'valid Vec<&'static str> {return self.link}
    fn get_mut<'valid>(&'valid mut self) -> &'valid mut Vec<&'static str> {return self.link}
}

//> SAME -> DERIVED STATE
const impl<'valid> DerivedState<'valid> for Same<'valid> {}

//> SAME -> CONVERT
const impl<'valid> Convert<'valid> for Same<'valid> {
    fn convert<Current: const State>(value: &'valid mut Current) -> Self {
        return Self {
            link: value.get_mut()
        }
    }
}


//^
//^ NAME
//^

//> NAME -> STRUCT
pub struct Name<'valid, const NAME: &'static str> {
    link: &'valid mut Vec<&'static str>
} 

//> NAME -> STATE
const impl<'this, const NAME: &'static str> State for Name<'this, NAME> {
    fn get<'valid>(&'valid self) -> &'valid Vec<&'static str> {return self.link}
    fn get_mut<'valid>(&'valid mut self) -> &'valid mut Vec<&'static str> {return self.link}
} 

//> NAME -> DROP
impl<'valid, const NAME: &'static str> Drop for Name<'valid, NAME> {
    fn drop(&mut self) {self.link.pop();}
}

//> NAME -> DERIVED STATE
const impl<'valid, const NAME: &'static str> DerivedState<'valid> for Name<'valid, NAME> {}

//> NAME -> CONVERT
const impl<'valid, const NAME: &'static str> Convert<'valid> for Name<'valid, NAME> {
    fn convert<Current: const State>(value: &'valid mut Current) -> Self {
        let link = value.get_mut();
        link.push(NAME);
        return Self {
            link: link
        }
    }
}