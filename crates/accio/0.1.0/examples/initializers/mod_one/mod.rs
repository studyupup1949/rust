use crate::TestMap;
use accio::*;

accio_emit!{
	mods {
		mod mod_one;
		// this is the catch:
		// since this code will be placed
		// into the caller (main.rs); all paths
		// are relative to the caller. so even though
		// the init function is RIGHT HERE; we cannot
		// just say init.
		use crate::mod_one::init as init_one;
	}
	initializers{
		init_one,
	}
}

pub fn init(m: &mut TestMap) {
	m.insert("one", 1);
}