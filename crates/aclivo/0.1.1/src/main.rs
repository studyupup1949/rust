
struct Server {
}

impl Server {
    fn create_dimension(&self, name: String) {
        println!("create_dimension {}", name);
    }

    fn create_element(&self, elname: String, dimname: String) {
        println!("the element {} was added to {} dimension", elname, dimname);
    }
}

fn new_server() -> Server {
    println!("server initialized");
    Server{}
}


fn main() {
    let server = new_server();
    server.create_dimension("month".to_string());
    server.create_dimension("sales".to_string());
    server.create_element("201801".to_string(), "month".to_string());
}
