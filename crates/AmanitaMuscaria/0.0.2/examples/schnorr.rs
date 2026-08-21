use AmanitaMuscaria::prelude::*;

fn main(){
    let phrase = BIP39API::generate(Language::English);
    let seed = phrase.derive_seed("Password1234", Language::English);

    let keypair = SchnorrKeypair::new(&seed.as_bytes());
    let signature = keypair.simple_sign(b"This is the Context",b"This message is being signed");
}