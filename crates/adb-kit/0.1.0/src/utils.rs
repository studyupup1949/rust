use crate::error::{ADBError, ADBResult};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};
use log::warn;
use regex::Regex;
use std::fs;
use rand::Rng;

/// 使用指数退避策略重试操作
pub fn retry_with_backoff<F, T>(max_retries: u32, initial_delay_ms: u64, f: F) -> ADBResult<T>
where
    F: Fn() -> ADBResult<T>,
{
    let mut retries = 0;
    let mut delay = initial_delay_ms;

    loop {
        match f() {
            Ok(result) => return Ok(result),
            Err(e) => {
                retries += 1;
                if retries > max_retries {
                    return Err(e);
                }

                warn!(
                    "操作失败 (重试 {}/{}), 延迟 {}ms: {}",
                    retries, max_retries, delay, e
                );

                std::thread::sleep(Duration::from_millis(delay));
                // 指数退避策略：下次延迟时间翻倍但不超过 10 秒
                delay = (delay * 2).min(10000);
            }
        }
    }
}

/// 带超时执行操作
pub fn with_timeout<F, T>(timeout_ms: u64, f: F) -> ADBResult<T>
where
    F: FnOnce() -> ADBResult<T> + Send + 'static,
    T: Send + 'static,
{
    let timeout = Duration::from_millis(timeout_ms);

    // 创建通道用于跨线程通信
    let (sender, receiver) = std::sync::mpsc::channel();

    // 在新线程中执行操作
    std::thread::spawn(move || {
        let result = f();
        let _ = sender.send(result);
    });

    // 等待结果或超时
    match receiver.recv_timeout(timeout) {
        Ok(result) => result,
        Err(_) => Err(ADBError::TimeoutError {
            message: "操作超时".to_string(),
            duration: timeout,
        }),
    }
}

/// 根据条件轮询等待
pub fn wait_with_polling<F, C>(
    timeout_ms: u64,
    poll_interval_ms: u64,
    condition_fn: F,
    callback: Option<C>,
) -> ADBResult<bool>
where
    F: Fn() -> ADBResult<bool>,
    C: Fn(u64),
{
    let start = Instant::now();
    let timeout = Duration::from_millis(timeout_ms);
    let interval = Duration::from_millis(poll_interval_ms);

    loop {
        // 检查是否超时
        let elapsed = start.elapsed();
        if elapsed > timeout {
            return Ok(false);
        }

        // 如果提供了回调函数，则执行
        if let Some(cb) = &callback {
            cb(elapsed.as_millis() as u64);
        }

        // 检查条件
        match condition_fn() {
            Ok(true) => return Ok(true),
            Ok(false) => {
                // 条件未满足，继续等待
                std::thread::sleep(interval);
            }
            Err(e) => {
                // 检查条件时出错
                warn!("检查条件时出错: {}", e);
                std::thread::sleep(interval);
            }
        }
    }
}

/// 解析 getprop 输出为属性 HashMap
pub fn parse_properties(output: &str) -> HashMap<String, String> {
    let mut properties = HashMap::new();
    let re = Regex::new(r"^\[([^\]]+)\]:\s*\[([^\]]*)\]$").unwrap_or_else(|_| {
        // 如果正则表达式无效，使用一个永远不会匹配的模式
        Regex::new(r"^$").unwrap()
    });

    for line in output.lines() {
        if let Some(caps) = re.captures(line.trim()) {
            if let (Some(key), Some(value)) = (caps.get(1), caps.get(2)) {
                properties.insert(key.as_str().to_string(), value.as_str().to_string());
            }
        }
    }

    properties
}

/// 清理不完整的分包
pub fn cleanup_partial_files(path_pattern: &str) -> ADBResult<()> {
    let pattern = format!("{}*", path_pattern);
    let glob_paths = match glob::glob(&pattern) {
        Ok(paths) => paths,
        Err(e) => {
            return Err(ADBError::FileError(format!(
                "无法匹配部分文件: {}",
                e
            )))
        }
    };

    for entry in glob_paths {
        match entry {
            Ok(path) => {
                if path.is_file() {
                    if let Err(e) = fs::remove_file(&path) {
                        warn!("无法删除临时文件 {:?}: {}", path, e);
                    }
                }
            }
            Err(e) => warn!("无法访问文件路径: {}", e),
        }
    }

    Ok(())
}

/// 创建临时目录
pub fn create_temp_dir_path(prefix: &str) -> ADBResult<PathBuf> {
    let temp_dir = std::env::temp_dir();

    // 生成随机字符串
    let random_string: String = rand::rng().sample_iter(&rand::distr::Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();

    let dir_name = format!("{}_{}", prefix, random_string);
    let full_path = temp_dir.join(dir_name);

    // 确保目录存在
    fs::create_dir_all(&full_path).map_err(|e| {
        ADBError::FileError(format!("无法创建临时目录: {}", e))
    })?;

    Ok(full_path)
}

/// 检查路径是否是有效的 APK 文件
pub fn is_valid_apk(path: &Path) -> bool {
    if !path.exists() || !path.is_file() {
        return false;
    }

    // 检查扩展名
    if let Some(ext) = path.extension() {
        return ext == "apk";
    }

    false
}

/// 获取字符串中的数字部分
pub fn extract_number(s: &str) -> Option<i32> {
    let re = Regex::new(r"\d+").ok()?;
    re.find(s)
        .map(|m| m.as_str())
        .and_then(|num_str| num_str.parse::<i32>().ok())
}

/// 格式化大小 (字节转换为 KB/MB/GB)
pub fn format_size(size_bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if size_bytes >= GB {
        format!("{:.2} GB", size_bytes as f64 / GB as f64)
    } else if size_bytes >= MB {
        format!("{:.2} MB", size_bytes as f64 / MB as f64)
    } else if size_bytes >= KB {
        format!("{:.2} KB", size_bytes as f64 / KB as f64)
    } else {
        format!("{} B", size_bytes)
    }
}

/// 解析命令行参数
pub fn parse_args(args: &[String]) -> HashMap<String, String> {
    let mut result = HashMap::new();
    let mut i = 0;

    while i < args.len() {
        if args[i].starts_with("--") {
            let key = args[i][2..].to_string();

            if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                result.insert(key, args[i + 1].clone());
                i += 2;
            } else {
                result.insert(key, "true".to_string());
                i += 1;
            }
        } else {
            i += 1;
        }
    }

    result
}

/// 获取迭代器中的第 N 行
pub fn get_line_at<'a>(lines: &mut std::str::Lines<'a>, index: usize) -> Option<&'a str> {
    lines.nth(index)
}

/// 安全地解析数字
pub fn parse_number<T: std::str::FromStr>(s: &str) -> Option<T> {
    s.trim().parse::<T>().ok()
}

/// 检查字符串是否包含任何给定的关键字
pub fn contains_any(s: &str, keywords: &[&str]) -> bool {
    for keyword in keywords {
        if s.contains(keyword) {
            return true;
        }
    }
    false
}

/// 将秒数转换为人类可读的时间格式 (HH:MM:SS)
pub fn format_duration(seconds: u64) -> String {
    let hours = seconds / 3600;
    let minutes = (seconds % 3600) / 60;
    let secs = seconds % 60;

    format!("{:02}:{:02}:{:02}", hours, minutes, secs)
}