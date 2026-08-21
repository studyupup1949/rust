use adb_rust::{ADB, prelude::*};

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

    // 截图
    let screenshot_path = "screenshot.png";
    println!("正在截图...");
    adb.take_screenshot_managed(device_id, screenshot_path)?;
    println!("截图已保存到: {}", screenshot_path);

    // 录制屏幕
    let recording_path = "screen_recording.mp4";
    println!("正在录制屏幕 (5 秒)...");
    adb.record_screen_managed(device_id, recording_path, 5, None)?;
    println!("录制已保存到: {}", recording_path);

    Ok(())
}
