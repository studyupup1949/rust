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

// Prepended Key returns two variables
// 1. Message (String)
// 2. Signature (Base64-encoded String)
fn aggregated_with_prepended_key(){
    let phrase = BIP39API::generate(Language::English);
    let seed = phrase.derive_seed("Password1234", Language::English);
    let keypair = BLSKeypair::new(&seed.as_bytes());
    let sig1 = keypair.sign_by_prepending_key("Hello World.");

    // 2
    let phrase2 = BIP39API::generate(Language::English);
    let seed2 = phrase2.derive_seed("Password1234", Language::English);
    let keypair2 = BLSKeypair::new(&seed2.as_bytes());
    let sig2 = keypair2.sign_by_prepending_key("Hello World.");


    let phrase3 = BIP39API::generate(Language::English);
    let seed3 = phrase3.derive_seed("Password1234", Language::English);
    let keypair3= BLSKeypair::new(&seed3.as_bytes());
    let sig3 = keypair3.sign_by_prepending_key("Hello World.");

    let final_sig = BLSKeypair::aggregate(&vec![sig1.1.clone(),sig2.1,sig3.1]);

    //println!("Keypair 1 (Signed Message): {}",sig1.0);
    //println!("Keypair 1 (Signature): {}",sig1.1.clone());

    let msg = VerifyBLSSignature::get_message_from_prepended_key(sig1.0.clone());
    println!("Message Without PK: {}",msg);

    let is_valid = VerifyBLSSignature::verify_prepended_keys(vec![sig1.0.clone(),sig2.0,sig3.0], final_sig);

    println!("Aggregate BLS Verify Prepended Keys Validility: {}",is_valid);
}