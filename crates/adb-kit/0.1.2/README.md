# ADB Kit

ADB Kit 是一个用于与 Android Debug Bridge (ADB) 进行交互的 Rust 库，提供了丰富的 API 以控制和管理 Android 设备。

## 特性

- 设备管理：列出、连接、断开设备
- 应用管理：安装、卸载、启动、停止应用
- 文件传输：向设备推送文件、从设备拉取文件
- 屏幕捕获：截图和屏幕录制
- 远程调试：启用 TCP/IP 连接和 Frida 服务器支持
- 并行操作：在多设备上并行执行命令
- 资源管理：自动清理临时文件

## 安装

在你的 `Cargo.toml` 中添加以下依赖:

```toml
[dependencies]
adb-kit = "0.1.0"
```

## 基本用法

```rust
use adb_rust::{ADB, ADBConfig, prelude::*};

fn main() -> ADBResult<()> {
    // 创建 ADB 实例
    let adb = ADB::new(None);
    
    // 列出设备
    let devices = adb.list_devices()?;
    println!("发现 {} 个设备", devices.len());
    
    for device in &devices {
        println!("设备 ID: {}, 状态: {}", device.id, device.status);
        
        if device.is_online() {
            // 获取 Android 版本
            let version = adb.get_prop(&device.id, "ro.build.version.release")?;
            println!("Android 版本: {}", version);
            
            // 列出已安装的应用
            let apps = adb.list_packages(&device.id, false, true)?;
            println!("已安装的第三方应用: {}", apps.len());
        }
    }
    
    Ok(())
}
```

## 应用管理

```rust
// 获取应用信息
let info = adb.get_package_info(&device_id, "com.example.app")?;
println!("应用版本: {:?}", info.version_name);

// 检查应用是否运行
let (running, pid) = adb.is_package_running(&device_id, "com.example.app")?;

// 启动应用
adb.start_app(&device_id, "com.example.app", None)?;

// 停止应用
adb.stop_app(&device_id, "com.example.app")?;

// 安装应用
adb.install_app(&device_id, "path/to/app.apk")?;

// 卸载应用
adb.uninstall_app(&device_id, "com.example.app")?;
```

## 文件传输

```rust
// 推送文件到设备
adb.push(&device_id, "local_file.txt", "/sdcard/remote_file.txt", None)?;

// 从设备拉取文件
adb.pull(&device_id, "/sdcard/remote_file.txt", "downloaded_file.txt", None)?;

// 检查文件是否存在
let exists = adb.file_exists(&device_id, "/sdcard/file.txt")?;

// 获取文件大小
let size = adb.get_file_size(&device_id, "/sdcard/file.txt")?;

// 创建目录
adb.create_directory(&device_id, "/sdcard/my_folder")?;

// 删除文件
adb.remove_path(&device_id, "/sdcard/file.txt", false)?;
```

## 屏幕捕获

```rust
// 截图
adb.take_screenshot(&device_id, "screenshot.png")?;

// 录制屏幕
adb.record_screen(&device_id, "recording.mp4", 10, Some("720x1280"))?;

// 使用资源管理器自动清理临时文件
adb.take_screenshot_managed(&device_id, "screenshot.png")?;
```

## 并行操作

```rust
// 在多台设备上并行执行 shell 命令
let results = adb.parallel_shell(&device_ids, "getprop ro.product.model");

// 在所有在线设备上执行操作
let results = adb.on_all_online_devices(|id| {
    adb.get_prop(id, "ro.build.version.release")
})?;

// 在多台设备上并行安装应用
let results = adb.parallel_install_app(&device_ids, "app.apk");
```

## 高级功能

```rust
// 使用自定义配置
let config = ADBConfigBuilder::default()
    .path("/custom/path/to/adb")
    .max_retries(5)
    .retry_delay(1500)
    .timeout(60000)
    .build();
    
let adb = ADB::new(Some(config));

// 启用远程调试
let addr = adb.enable_remote_debugging(&device_id, 5555)?;
println!("可以使用 'adb connect {}' 连接", addr);

// 启动 Frida 服务器
adb.start_frida_server(&device_id, "./frida-server", 27042, None, Some(true))?;
```

## 完整示例

参见 [examples](examples/) 目录获取更多示例。

## 贡献

欢迎贡献！请随时提交问题或拉取请求。

## 许可证

MIT