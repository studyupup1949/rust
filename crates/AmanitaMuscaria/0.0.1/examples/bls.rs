use AmanitaMuscaria::prelude::*;

fn main(){
    let phrase = BIP39API::generate(Language::English);
    let seed = phrase.derive_seed("Password1234", Language::English);

    let keypair = BLSKeypair::new(&seed.as_bytes());

    let pk = keypair.return_pk_as_hex();

    let sig = keypair.sign("Hello World.");

    let is_valid: bool = VerifyBLSSignature::verify_with_hex(vec![pk], vec![b"Hello World."], sig);

    assert!(is_valid);
}

fn aggregated(){
    let phrase = BIP39API::generate(Language::English);
    let seed = phrase.derive_seed("Password1234", Language::English);

    let keypair = BLSKeypair::new(&seed.as_bytes());

    let pk = keypair.return_pk_as_hex();

    let sig = keypair.sign("Message 1");


    let phrase2 = BIP39API::generate(Language::English);
    let seed2 = phrase2.derive_seed("Password123", Language::English);

    let keypair2 = BLSKeypair::new(&seed2.as_bytes());

    let pk2 = keypair2.return_pk_as_hex();

    let sig2 = keypair2.sign("Message 2");



    let phrase3 = BIP39API::generate(Language::English);
    let seed3 = phrase3.derive_seed("Password1234", Language::English);

    let keypair3 = BLSKeypair::new(&seed3.as_bytes());

    let pk3 = keypair3.return_pk_as_hex();

    let sig3 = keypair3.sign("Message 3");

    let output = BLSKeypair::aggregate(&vec![sig,sig2,sig3]);

    let is_valid = VerifyBLSSignature::verify_with_hex(vec![pk,pk2,pk3], vec![b"Message 1",b"Message 2",b"Message 3"], output);

    assert!(is_valid);
}