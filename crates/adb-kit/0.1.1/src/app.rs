use crate::device::ADB;
use crate::error::{ADBError, ADBResult};
use log::{debug, info, warn};
use regex::Regex;
use std::process::Command;
use std::str::FromStr;
use std::time::{Duration, Instant};

/// 包信息结构体
#[derive(Debug, Clone)]
pub struct PackageInfo {
    pub package_name: String,
    pub version_name: Option<String>,
    pub version_code: Option<i32>,
    pub install_time: Option<String>,
    pub update_time: Option<String>,
    pub uid: Option<i32>,
    pub target_sdk: Option<i32>,
    pub min_sdk: Option<i32>,
    pub flags: Vec<String>,
    pub permissions: Vec<String>,
    pub activities: Vec<String>,
    pub services: Vec<String>,
    pub install_source: Option<String>,
    pub raw_data: Option<String>,
}

impl PackageInfo {
    /// 创建新的包信息实例
    pub fn new(package_name: &str) -> Self {
        Self {
            package_name: package_name.to_string(),
            version_name: None,
            version_code: None,
            install_time: None,
            update_time: None,
            uid: None,
            target_sdk: None,
            min_sdk: None,
            flags: Vec::new(),
            permissions: Vec::new(),
            activities: Vec::new(),
            services: Vec::new(),
            install_source: None,
            raw_data: None,
        }
    }

    /// 创建一个 PackageInfo 构建器
    pub fn builder(package_name: &str) -> crate::app::PackageInfoBuilder {
        crate::app::PackageInfoBuilder::new(package_name)
    }
}

/// 包信息构建器
#[derive(Debug)]
pub struct PackageInfoBuilder {
    info: PackageInfo,
}

impl PackageInfoBuilder {
    pub fn new(package_name: &str) -> Self {
        Self {
            info: PackageInfo::new(package_name),
        }
    }

    pub fn with_version_name(mut self, version: &str) -> Self {
        self.info.version_name = Some(version.to_string());
        self
    }

    pub fn with_version_code(mut self, code: i32) -> Self {
        self.info.version_code = Some(code);
        self
    }

    pub fn with_install_time(mut self, time: &str) -> Self {
        self.info.install_time = Some(time.to_string());
        self
    }

    pub fn with_update_time(mut self, time: &str) -> Self {
        self.info.update_time = Some(time.to_string());
        self
    }

    pub fn with_uid(mut self, uid: i32) -> Self {
        self.info.uid = Some(uid);
        self
    }

    pub fn with_target_sdk(mut self, sdk: i32) -> Self {
        self.info.target_sdk = Some(sdk);
        self
    }

    pub fn with_min_sdk(mut self, sdk: i32) -> Self {
        self.info.min_sdk = Some(sdk);
        self
    }

    pub fn add_flag(mut self, flag: &str) -> Self {
        self.info.flags.push(flag.to_string());
        self
    }

    pub fn add_permission(mut self, permission: &str) -> Self {
        self.info.permissions.push(permission.to_string());
        self
    }

    pub fn add_activity(mut self, activity: &str) -> Self {
        self.info.activities.push(activity.to_string());
        self
    }

    pub fn add_service(mut self, service: &str) -> Self {
        self.info.services.push(service.to_string());
        self
    }

    pub fn with_install_source(mut self, source: &str) -> Self {
        self.info.install_source = Some(source.to_string());
        self
    }

    pub fn with_raw_data(mut self, data: &str) -> Self {
        self.info.raw_data = Some(data.to_string());
        self
    }

    pub fn build(self) -> PackageInfo {
        self.info
    }
}

impl ADB {
    /// 获取包信息 (增强版本)
    pub fn get_package_info(&self, device_id: &str, package_name: &str) -> ADBResult<PackageInfo> {
        self.get_package_info_enhanced(device_id, package_name)
    }

    /// 获取包信息 (增强版本)
    pub fn get_package_info_enhanced(
        &self,
        device_id: &str,
        package_name: &str,
    ) -> ADBResult<PackageInfo> {
        let command = format!("dumpsys package {}", package_name);
        let output = self.shell(device_id, &command)?;

        // 存储原始输出以便调试和完整访问
        let mut info = PackageInfo::new(package_name);
        info.raw_data = Some(output.clone());

        // 使用正则表达式解析更多信息
        if let Some(re_version) = Regex::new(r"versionName=([^\s]+)").ok() {
            if let Some(caps) = re_version.captures(&output) {
                if let Some(ver) = caps.get(1) {
                    info.version_name = Some(ver.as_str().to_string());
                }
            }
        }

        if let Some(re_code) = Regex::new(r"versionCode=(\d+)").ok() {
            if let Some(caps) = re_code.captures(&output) {
                if let Some(code) = caps.get(1) {
                    if let Ok(code_int) = i32::from_str(code.as_str()) {
                        info.version_code = Some(code_int);
                    }
                }
            }
        }

        // 提取首次安装时间
        if let Some(re_install) = Regex::new(r"firstInstallTime=([^\s]+)").ok() {
            if let Some(caps) = re_install.captures(&output) {
                if let Some(time) = caps.get(1) {
                    info.install_time = Some(time.as_str().to_string());
                }
            }
        }

        // 提取最后更新时间
        if let Some(re_update) = Regex::new(r"lastUpdateTime=([^\s]+)").ok() {
            if let Some(caps) = re_update.captures(&output) {
                if let Some(time) = caps.get(1) {
                    info.update_time = Some(time.as_str().to_string());
                }
            }
        }

        // 提取 UID
        if let Some(re_uid) = Regex::new(r"userId=(\d+)").ok() {
            if let Some(caps) = re_uid.captures(&output) {
                if let Some(uid) = caps.get(1) {
                    if let Ok(uid_int) = i32::from_str(uid.as_str()) {
                        info.uid = Some(uid_int);
                    }
                }
            }
        }

        // 提取 SDK 版本信息
        if let Some(re_target_sdk) = Regex::new(r"targetSdk=(\d+)").ok() {
            if let Some(caps) = re_target_sdk.captures(&output) {
                if let Some(sdk) = caps.get(1) {
                    if let Ok(sdk_int) = i32::from_str(sdk.as_str()) {
                        info.target_sdk = Some(sdk_int);
                    }
                }
            }
        }

        if let Some(re_min_sdk) = Regex::new(r"minSdk=(\d+)").ok() {
            if let Some(caps) = re_min_sdk.captures(&output) {
                if let Some(sdk) = caps.get(1) {
                    if let Ok(sdk_int) = i32::from_str(sdk.as_str()) {
                        info.min_sdk = Some(sdk_int);
                    }
                }
            }
        }

        // 提取安装来源
        if let Some(re_install_source) = Regex::new(r"installerPackageName=([^\s]+)").ok() {
            if let Some(caps) = re_install_source.captures(&output) {
                if let Some(source) = caps.get(1) {
                    info.install_source = Some(source.as_str().to_string());
                }
            }
        }

        // 提取权限
        let lines = output.lines().collect::<Vec<&str>>();
        let mut in_permissions = false;

        for line in &lines {
            if line.contains("requested permissions:") {
                in_permissions = true;
                continue;
            } else if in_permissions && line.trim().is_empty() {
                in_permissions = false;
                continue;
            }

            if in_permissions && line.contains(": granted=") {
                if let Some(perm) = line.split(':').next() {
                    let perm = perm.trim();
                    if !perm.is_empty() {
                        info.permissions.push(perm.to_string());
                    }
                }
            }
        }

        // 提取 Activities
        let mut in_activities = false;
        for line in &lines {
            if line.contains("Activity Resolver Table:") {
                in_activities = true;
                continue;
            } else if in_activities && line.trim().is_empty() {
                in_activities = false;
                continue;
            }

            if in_activities && line.contains(package_name) {
                if let Some(activity) = Regex::new(r"/([^/\s]+)")
                    .ok()
                    .and_then(|re| re.captures(line))
                    .and_then(|caps| caps.get(1))
                    .map(|m| m.as_str())
                {
                    info.activities.push(activity.to_string());
                }
            }
        }

        Ok(info)
    }

    /// 检查包是否运行
    pub fn is_package_running(
        &self,
        device_id: &str,
        package_name: &str,
    ) -> ADBResult<(bool, Option<i32>)> {
        // 先尝试使用更准确的方法检查是否运行中
        if let Ok(Some(pid)) = self.get_pid(device_id, package_name) {
            return Ok((true, Some(pid)));
        }

        // 备用方法 - 使用 ps 命令
        let command = format!("ps -A | grep -i {}", package_name);
        let output = self.shell(device_id, &command)?;

        // 如果找到输出，则尝试提取 PID
        if !output.trim().is_empty() {
            let lines = output.lines().next();
            if let Some(line) = lines {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() > 1 {
                    if let Ok(pid) = i32::from_str(parts[1]) {
                        debug!("找到 PID: {}", pid);
                        return Ok((true, Some(pid)));
                    }
                }
            }
            return Ok((true, None));
        }

        // 最后的检查 - 使用 dumpsys
        let command = format!("dumpsys activity services | grep -i {}", package_name);
        let output = self.shell(device_id, &command)?;
        if !output.trim().is_empty() {
            return Ok((true, None));
        }

        Ok((false, None))
    }

    /// 获取进程 ID
    pub fn get_pid(&self, device_id: &str, package_name: &str) -> ADBResult<Option<i32>> {
        // 尝试多种方法获取 PID
        let methods = [
            // 方法1: 使用 pidof (适用于较新的 Android 系统)
            format!("pidof {}", package_name),
            // 方法2: 使用 ps 带有 grep 过滤 (兼容不同 Android 版本)
            format!("ps -A | grep {} | grep -v grep", package_name),
            // 方法3: 使用 ps -o 带有 NAME 过滤 (Android 8+)
            format!("ps -A -o PID,NAME | grep {} | grep -v grep", package_name),
            // 方法4: 使用 ps -o 带有 CMDLINE 过滤 (用于某些特殊情况)
            format!(
                "ps -A -o PID,CMDLINE | grep {} | grep -v grep",
                package_name
            ),
        ];

        for method in &methods {
            let output = self.shell(device_id, method);

            if let Ok(pid_str) = output {
                if pid_str.trim().is_empty() {
                    continue;
                }

                // 尝试解析 PID
                if let Some(line) = pid_str.lines().next() {
                    let parts: Vec<&str> = line.split_whitespace().collect();

                    // 处理不同命令的不同输出格式
                    if parts.is_empty() {
                        continue;
                    }

                    // pidof 直接返回 PID
                    if method.starts_with("pidof") {
                        if let Ok(pid) = i32::from_str(parts[0]) {
                            debug!("通过 pidof 获取 PID: {}", pid);
                            return Ok(Some(pid));
                        }
                    }
                    // ps 命令通常在第二列返回 PID
                    else if parts.len() > 1 {
                        if let Ok(pid) = i32::from_str(parts[0]) {
                            debug!("通过 ps 获取 PID: {}", pid);
                            return Ok(Some(pid));
                        }
                    }
                }
            }
        }

        debug!("无法找到包 {} 的 PID", package_name);
        Ok(None)
    }

    /// 启动一个应用并等待直到完全启动
    pub fn start_app_and_wait(
        &self,
        device_id: &str,
        package_name: &str,
        activity: Option<&str>,
        timeout_secs: Option<u64>,
    ) -> ADBResult<bool> {
        let timeout = timeout_secs.unwrap_or(30);
        let start_time = Instant::now();

        // 构建启动命令
        let command = if let Some(act) = activity {
            format!("am start -W -n {}/{}", package_name, act)
        } else {
            format!(
                "monkey -p {} -c android.intent.category.LAUNCHER 1",
                package_name
            )
        };

        // 执行启动命令
        let output = self.shell(device_id, &command)?;

        // 检查是否有即时错误
        if output.contains("Error") || output.contains("Exception") || output.contains("failed") {
            debug!("应用程序启动失败: {}", output);
            return Ok(false);
        }

        // 等待应用完全启动
        info!("等待应用 {} 完全启动...", package_name);

        loop {
            if start_time.elapsed().as_secs() > timeout {
                warn!("等待应用启动超时 ({} 秒)", timeout);
                return Ok(false);
            }

            // 检查应用是否在前台运行
            let current_app_cmd = "dumpsys window windows | grep -E 'mCurrentFocus'";
            let current_app = self.shell(device_id, current_app_cmd)?;

            if current_app.contains(package_name) {
                debug!("应用 {} 已成功启动并处于前台", package_name);
                return Ok(true);
            }

            // 短暂休眠后再次检查
            std::thread::sleep(Duration::from_millis(500));
        }
    }

    /// 启动应用程序
    pub fn start_app(
        &self,
        device_id: &str,
        package_name: &str,
        activity: Option<&str>,
    ) -> ADBResult<bool> {
        let command = if let Some(act) = activity {
            format!("am start -n {}/{}", package_name, act)
        } else {
            format!(
                "monkey -p {} -c android.intent.category.LAUNCHER 1",
                package_name
            )
        };

        let output = self.shell(device_id, &command)?;

        // 分析输出以确定启动是否成功
        if output.contains("Error") || output.contains("Exception") || output.contains("failed") {
            debug!("启动应用程序失败: {}", output);
            return Ok(false);
        } else {
            debug!("应用程序启动命令执行成功");
            return Ok(true);
        }
    }

    /// 强制停止应用程序
    pub fn stop_app(&self, device_id: &str, package_name: &str) -> ADBResult<()> {
        let command = format!("am force-stop {}", package_name);
        self.shell(device_id, &command)?;
        Ok(())
    }

    /// 安装应用程序
    pub fn install_app(&self, device_id: &str, apk_path: &str) -> ADBResult<()> {
        self.with_retry(|| {
            let mut cmd = Command::new(&self.config.path);
            if !device_id.is_empty() {
                cmd.arg("-s").arg(device_id);
            }

            let output = cmd
                .arg("install")
                .arg("-r") // Replace existing app
                .arg(apk_path)
                .output()
                .map_err(|e| ADBError::CommandError(format!("无法安装 APK: {}", e)))?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if !output.status.success() || stdout.contains("Failure") || stderr.contains("Failure")
            {
                let error_msg = if stdout.contains("Failure") {
                    format!("APK 安装失败: {}", stdout)
                } else if !stderr.is_empty() {
                    format!("APK 安装失败: {}", stderr)
                } else {
                    "APK 安装失败，未知错误".to_string()
                };

                return Err(ADBError::CommandError(error_msg));
            }

            debug!("成功安装 APK: {}", apk_path);
            Ok(())
        })
    }

    /// 卸载应用程序
    pub fn uninstall_app(&self, device_id: &str, package_name: &str) -> ADBResult<()> {
        self.with_retry(|| {
            let mut cmd = Command::new(&self.config.path);
            if !device_id.is_empty() {
                cmd.arg("-s").arg(device_id);
            }

            let output = cmd
                .arg("uninstall")
                .arg(package_name)
                .output()
                .map_err(|e| {
                    ADBError::CommandError(format!("无法卸载应用: {}", e))
                })?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if !output.status.success() || stdout.contains("Failure") || stderr.contains("Failure")
            {
                let error_msg = if stdout.contains("Failure") {
                    format!("应用卸载失败: {}", stdout)
                } else if !stderr.is_empty() {
                    format!("应用卸载失败: {}", stderr)
                } else {
                    "应用卸载失败，未知错误".to_string()
                };

                return Err(ADBError::CommandError(error_msg));
            }

            debug!("成功卸载应用: {}", package_name);
            Ok(())
        })
    }

    /// 智能卸载应用(清除数据和缓存)
    pub fn uninstall_app_smart(
        &self,
        device_id: &str,
        package_name: &str,
        keep_data: bool,
    ) -> ADBResult<()> {
        // 首先停止应用
        let _ = self.stop_app(device_id, package_name);

        // 如果需要保留数据，先清除缓存
        if keep_data {
            let _ = self.shell(device_id, &format!("pm clear {}", package_name));
        }

        // 执行卸载
        self.with_retry(|| {
            let mut cmd = Command::new(&self.config.path);
            if !device_id.is_empty() {
                cmd.arg("-s").arg(device_id);
            }

            let mut args = vec!["uninstall"];

            if keep_data {
                args.push("-k"); // 保留数据和缓存文件
            }

            args.push(package_name);

            for arg in args {
                cmd.arg(arg);
            }

            let output = cmd
                .output()
                .map_err(|e| ADBError::CommandError(format!("卸载包失败: {}", e)))?;

            let stdout = String::from_utf8_lossy(&output.stdout).to_string();
            let stderr = String::from_utf8_lossy(&output.stderr).to_string();

            if !output.status.success() || stdout.contains("Failure") || stderr.contains("Failure")
            {
                let error_msg = if stdout.contains("Failure") {
                    format!("卸载应用失败: {}", stdout)
                } else if !stderr.is_empty() {
                    format!("卸载应用失败: {}", stderr)
                } else {
                    "卸载应用失败，未知错误".to_string()
                };

                return Err(ADBError::CommandError(error_msg));
            }

            debug!("成功卸载应用: {}", package_name);
            Ok(())
        })
    }

    /// 获取设备上已安装的应用列表
    pub fn list_packages(
        &self,
        device_id: &str,
        only_system: bool,
        only_third_party: bool,
    ) -> ADBResult<Vec<String>> {
        let mut command = "pm list packages".to_string();

        if only_system {
            command.push_str(" -s");
        } else if only_third_party {
            command.push_str(" -3");
        }

        let output = self.shell(device_id, &command)?;
        let mut packages = Vec::new();

        for line in output.lines() {
            if line.starts_with("package:") {
                let package = line.trim_start_matches("package:").trim();
                packages.push(package.to_string());
            }
        }

        Ok(packages)
    }
}