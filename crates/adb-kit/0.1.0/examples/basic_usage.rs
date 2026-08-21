use adb_rust::{ADB, ADBConfig, prelude::*};

fn main() -> ADBResult<()> {
    // 创建配置
    let config = ADBConfig::default();

    // 创建 ADB 实例
    let adb = ADB::new(Some(config));

    // 检查 ADB 是否可用
    match adb.check_adb() {
        Ok(version) => println!("ADB 版本: {}", version),
        Err(e) => {
            eprintln!("ADB 不可用: {}", e);
            return Err(e);
        }
    }

    // 列出连接的设备
    let devices = adb.list_devices()?;
    println!("发现 {} 个设备:", devices.len());

    for device in &devices {
        println!("  ID: {}, 名称: {}, 状态: {}", device.id, device.name, device.status);

        if device.is_online() {
            // 获取设备属性
            let android_version = adb.get_prop(&device.id, "ro.build.version.release")?;
            println!("  Android 版本: {}", android_version);

            // 列出已安装的第三方应用
            let apps = adb.list_packages(&device.id, false, true)?;
            println!("  已安装的第三方应用数量: {}", apps.len());

            if !apps.is_empty() {
                let app = &apps[0];
                println!("  获取应用信息: {}", app);

                // 获取应用信息
                if let Ok(info) = adb.get_package_info(&device.id, app) {
                    if let Some(version) = &info.version_name {
                        println!("    版本: {}", version);
                    }
                    println!("    权限数量: {}", info.permissions.len());
                }

                // 检查应用是否在运行
                let (running, pid) = adb.is_package_running(&device.id, app)?;
                if running {
                    println!("    应用正在运行, PID: {:?}", pid);
                } else {
                    println!("    应用未运行");
                }
            }
        }
    }

    Ok(())
}