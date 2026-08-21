use adb_lib::Adb;

#[test]
fn test_devices() {
    let adb = Adb::new("adb");
    match adb.devices() {
        Ok(output) => assert!(output.contains("List of devices")),
        Err(e) => panic!("Error: {}", e),
    }
}

// #[test]
// fn test_install() {
//     let adb = Adb::new("adb");
//     match adb.install("path/to/app.apk") {
//         Ok(output) => println!("Install output: {}", output),
//         Err(e) => panic!("Error: {}", e),
//     }
// }

// #[test]
// fn test_uninstall() {
//     let adb = Adb::new("adb");
//     match adb.uninstall("com.example.app") {
//         Ok(output) => println!("Uninstall output: {}", output),
//         Err(e) => panic!("Error: {}", e),
//     }
// }
