use AmanitaMuscaria::prelude::*;

//==========KEYPAIRS==========//

fn main(){
    // Generate Keypair
    let keypair = Falcon1024Keypair::generate();
    
    // Sign Message
    let sig = keypair.sign("This Message Is Signed");

    // Verify Signature
    let is_valid = sig.verify();
}

fn return_values(){
    let keypair = Falcon1024Keypair::generate();

    let pk_hex = keypair.return_public_key_as_hex();
    let pk_bytes = keypair.return_public_key_as_bytes();

    let sk_hex = keypair.return_secret_key_as_hex();
    let sk_bytes = keypair.return_secret_key_as_bytes();
}

// An address is the 48 byte hash of the public key
fn return_address(){
    let keypair = Falcon1024Keypair::generate();

    let address = keypair.return_address();
    let address_with_key = keypair.return_address_using_key("Key 123");
}

//==========SIGNATURES==========//

fn return_signature_values(){
    let keypair: Falcon1024Keypair = Falcon1024Keypair::generate();

    let sig: Falcon1024Signature = keypair.sign("Hello World.");

    let is_valid_basic = sig.verify();

    let signature_base64: String = sig.return_signature_in_base64();

    let message: String = sig.return_message();

    let pk_hex: String =  sig.return_public_key_as_hex();

    // Verifies signature using inputs
    let is_valid: bool = Falcon1024Signature::verify_with_input(pk_hex,signature_base64,message);
}