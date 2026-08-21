cargo +nightly test --features "syncall" -- --nocapture test_syncall
cargo +nightly test --features "syncall" -- --nocapture test_atomic_call
cargo +nightly test --features "net" -- --nocapture test_get_interface_list
cargo +nightly test --features "dev" -- --nocapture test_find_usb_device
cargo +nightly test --features "net" -- --nocapture test_get_route_table
cargo +nightly test --features "net" -- --nocapture test_ping