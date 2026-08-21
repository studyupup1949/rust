use crate::TestMap;
use accio::*;

accio_emit!{
	mods {
		mod mod_two;
		use crate::mod_two::init as init_two;
	}
	initializers{
		init_two,
	}
}

pub fn init(m: &mut TestMap) {
	m.insert("two", 2);
}
