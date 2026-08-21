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

    // 列出已安装的第三方应用
    let apps = adb.list_packages(device_id, false, true)?;
    println!("已安装的第三方应用数量: {}", apps.len());

    if !apps.is_empty() {
        let app = &apps[0];
        println!("选择应用: {}", app);

        // 获取应用信息
        let info = adb.get_package_info(device_id, app)?;
        println!("应用信息:");
        println!("  版本名称: {:?}", info.version_name);
        println!("  版本代码: {:?}", info.version_code);
        println!("  安装时间: {:?}", info.install_time);
        println!("  权限数量: {}", info.permissions.len());

        // 检查应用是否在运行
        let (running, pid) = adb.is_package_running(device_id, app)?;
        println!("应用运行状态: {}, PID: {:?}", running, pid);

        // 启动应用
        if !running {
            println!("启动应用...");
            let success = adb.start_app(device_id, app, None)?;
            println!("启动结果: {}", success);

            // 等待应用启动
            std::thread::sleep(std::time::Duration::from_secs(2));

            // 再次检查应用是否在运行
            let (running, pid) = adb.is_package_running(device_id, app)?;
            println!("应用运行状态: {}, PID: {:?}", running, pid);
        }

        // 停止应用
        if running {
            println!("停止应用...");
            adb.stop_app(device_id, app)?;

            // 等待应用停止
            std::thread::sleep(std::time::Duration::from_secs(1));

            // 再次检查应用是否在运行
            let (running, _) = adb.is_package_running(device_id, app)?;
            println!("应用运行状态: {}", running);
        }
    }

    Ok(())
}