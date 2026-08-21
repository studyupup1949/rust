pub fn public_function() {
    println!("Hello I am a public function");
}

fn private_function() {
    println!("Hello I am a private function");
}

pub fn indirect_access() {
    print!("Accessing a private library using a public function");

    private_function();
}

fn main() {
    public_function();
    indirect_access();
}
