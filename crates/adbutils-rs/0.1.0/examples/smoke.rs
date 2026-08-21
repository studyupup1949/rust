//! Minimal smoke test against a real adb server: prints server version and the
//! device list. Run with: `cargo run --example smoke`.

#[tokio::main]
async fn main() -> adbutils::Result<()> {
    let adb = adbutils::adb();
    println!("server version: {}", adb.server_version().await?);
    let devices = adb.device_list().await?;
    println!("device count: {}", devices.len());
    for d in &devices {
        let serial = d.serial().unwrap_or("?");
        let state = d.get_state().await?;
        let model = d.shell("getprop ro.product.model").await?;
        println!("  {serial}  state={state}  model={model}");
    }
    Ok(())
}
