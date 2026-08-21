use adar_registry::prelude::*;

trait EndPoint {
    fn execute(&self);
}

struct GetUser;

impl EndPoint for GetUser {
    fn execute(&self) {
        println!("Getting user");
    }
}

fn main() {
    let registry = RegistryMap::<&'static str, Box<dyn EndPoint + Send + Sync + 'static>>::new();
    let _entry = registry.register("get_user", Box::new(GetUser));

    registry.read().get(&"get_user").unwrap().execute();
}
