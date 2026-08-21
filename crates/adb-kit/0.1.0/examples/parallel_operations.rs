use adb_rust::{ADB, prelude::*};

fn main() -> ADBResult<()> {
    let adb = ADB::new(None);

    // 列出设备
    let devices = adb.list_devices()?;
    println!("发现 {} 个设备", devices.len());

    if devices.len() < 2 {
        println!("此示例需要至少 2 个设备");
        return Ok(());
    }

    // 获取设备 ID 列表
    let device_ids: Vec<&str> = devices.iter().map(|d| d.id.as_str()).collect();

    // 并行执行 shell 命令
    println!("并行执行 shell 命令...");
    let shell_results = adb.parallel_shell(&device_ids, "getprop ro.product.model");

    for (id, result) in &shell_results {
        match result {
            Ok(output) => println!("设备 {}: 型号 = {}", id, output.trim()),
            Err(e) => println!("设备 {}: 错误 = {}", id, e),
        }
    }

    // 检查多个设备是否在线
    println!("检查设备在线状态...");
    let online_devices = adb.filter_online_devices(&device_ids)?;
    println!("在线设备: {}", online_devices.join(", "));

    // 在所有在线设备上获取电池信息
    println!("获取所有设备的电池信息...");
    let battery_results = adb.on_all_online_devices(|device_id| {
        adb.shell(device_id, "dumpsys battery")
    })?;

    for (id, result) in battery_results {
        match result {
            Ok(_) => println!("设备 {}: 已获取电池信息", id),
            Err(e) => println!("设备 {}: 获取电池信息失败 = {}", id, e),
        }
    }

    Ok(())
}