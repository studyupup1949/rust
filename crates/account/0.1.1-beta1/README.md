Account 

---

Provides a simple tool library for managing web3 accounts.
It provides a set of APIs for account management, including account creation, account import, account export, account deletion, account balance query, account transfer, etc.

### Usage

```rust
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
```