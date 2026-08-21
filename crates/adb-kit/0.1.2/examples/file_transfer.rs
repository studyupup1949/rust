use adb_kit::{ADB, transfer::TransferOptions, prelude::*};
use std::path::Path;

fn main() -> ADBResult<()> {
    let adb = ADB::new(None);

    // 列出设备
    let devices = adb.list_devices()?;
    if devices.is_empty() {
        println!("没有连接的设备");
        return Ok(());
    }

    let device_id = &devices[0].id;
    println!("使用设备: {}", device_id);

    // 推送文件
    let local_file = "test_file.txt";
    let device_path = "/sdcard/test_file.txt";

    // 创建测试文件
    if !Path::new(local_file).exists() {
        std::fs::write(local_file, "这是一个测试文件内容")?;
    }

    // 定义传输选项
    let options = TransferOptions {
        ..Default::default()
    };

    // 推送文件
    println!("推送文件: {} -> {}", local_file, device_path);
    adb.push(device_id, local_file, device_path, Some(options.clone()))?;

    // 检查文件是否存在
    let exists = adb.file_exists(device_id, device_path)?;
    println!("设备上文件存在: {}", exists);

    // 获取文件大小
    if exists {
        let size = adb.get_file_size(device_id, device_path)?;
        println!("文件大小: {} 字节", size);
    }

    // 拉取文件
    let local_output = "downloaded_test_file.txt";
    println!("拉取文件: {} -> {}", device_path, local_output);
    adb.pull(device_id, device_path, local_output, Some(options))?;

    // 删除设备上的文件
    println!("删除设备上的文件: {}", device_path);
    adb.remove_path(device_id, device_path, false)?;

    Ok(())
}