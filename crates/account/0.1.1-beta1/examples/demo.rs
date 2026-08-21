use account::account::Account;

fn main() {
    // create account
    let password = None;
    let account = Account::new(password);

    // using a child path to sign and verify
    let child_path = "m/44'/0'/0'/0/0'";
    // sign and verify
    let example_msg = b"Hello, world!";
    let sign_message = account.sign(child_path, example_msg);
    account.verify(child_path, example_msg, &sign_message.unwrap());
}
