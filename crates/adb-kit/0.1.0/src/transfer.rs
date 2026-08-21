use crate::device::ADB;
use crate::error::{ADBError, ADBResult};
use log::{debug, info};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::Path;
use std::process::Command;

/// 文件传输选项
#[derive(Debug, Clone)]
pub struct TransferOptions {
    // 共用选项
    pub compression: bool,                     // 启用压缩
    pub compression_algorithm: Option<String>, // 压缩算法: "any", "none", "brotli", "lz4", "zstd"

    // push 专用选项
    pub sync: bool,    // 使用 sync 模式进行传输 (--sync)
    pub dry_run: bool, // 干运行，不实际存储到文件系统 (-n)

    // pull 专用选项
    pub preserve_timestamp: bool, // 保留文件时间戳和模式 (-a)

    // 内部选项，不直接映射到 ADB 命令参数
    pub chunk_size: usize, // 分块大小(单位:字节)
}

impl Default for TransferOptions {
    fn default() -> Self {
        TransferOptions {
            compression: false,
            compression_algorithm: None,
            sync: false,
            dry_run: false,
            preserve_timestamp: false,
            chunk_size: 65536, // 64KB
        }
    }
}

impl ADB {
    /// 文件拉取
    pub fn pull(
        &self,
        device_id: &str,
        device_path: &str,
        local_path: &str,
        options: Option<TransferOptions>,
    ) -> ADBResult<()> {
        let options = options.unwrap_or_default();

        self.with_retry(|| {
            let mut cmd = Command::new(&self.config.path);

            // 如果指定了设备 ID 则添加
            if !device_id.is_empty() {
                cmd.arg("-s").arg(device_id);
            }

            cmd.arg("pull");

            // 添加传输选项
            if options.preserve_timestamp {
                cmd.arg("-a");
            }

            // 处理压缩选项
            if options.compression {
                if let Some(algorithm) = &options.compression_algorithm {
                    cmd.arg("-z").arg(algorithm);
                } else {
                    cmd.arg("-z").arg("any");
                }
            } else {
                cmd.arg("-Z");
            }

            // 设置源路径和目标路径
            cmd.arg(device_path).arg(local_path);

            info!("开始从设备拉取文件: {} -> {}", device_path, local_path);
            let output = cmd
                .output()
                .map_err(|e| ADBError::CommandError(format!("执行 ADB pull 命令失败: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!(
                    "ADB pull 命令失败: {}",
                    stderr
                )));
            }

            debug!("成功拉取文件 {} 到 {}", device_path, local_path);
            Ok(())
        })
    }

    /// 文件推送
    pub fn push(
        &self,
        device_id: &str,
        local_path: &str,
        device_path: &str,
        options: Option<TransferOptions>,
    ) -> ADBResult<()> {
        let options = options.unwrap_or_default();

        self.with_retry(|| {
            let mut cmd = Command::new(&self.config.path);

            // 如果指定了设备 ID 则添加
            if !device_id.is_empty() {
                cmd.arg("-s").arg(device_id);
            }

            cmd.arg("push");

            // 添加传输选项
            if options.sync {
                cmd.arg("--sync");
            }

            if options.dry_run {
                cmd.arg("-n");
            }

            // 处理压缩选项
            if options.compression {
                if let Some(algorithm) = &options.compression_algorithm {
                    cmd.arg("-z").arg(algorithm);
                } else {
                    cmd.arg("-z").arg("any");
                }
            } else {
                cmd.arg("-Z");
            }

            // 设置源路径和目标路径
            cmd.arg(local_path).arg(device_path);

            info!("开始向设备推送文件: {} -> {}", local_path, device_path);
            let output = cmd
                .output()
                .map_err(|e| ADBError::CommandError(format!("执行 ADB push 命令失败: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(ADBError::CommandError(format!(
                    "ADB push 命令失败: {}",
                    stderr
                )));
            }

            debug!("成功推送文件 {} 到 {}", local_path, device_path);
            Ok(())
        })
    }

    /// 分块推送大文件
    pub fn push_large_file(
        &self,
        device_id: &str,
        local_path: &str,
        device_path: &str,
        options: Option<TransferOptions>,
    ) -> ADBResult<()> {
        let options = options.unwrap_or_default();
        let chunk_size = options.chunk_size;

        // 确保文件存在
        let file_path = Path::new(local_path);
        if !file_path.exists() || !file_path.is_file() {
            let error_msg = format!("文件不存在: {}", local_path);
            return Err(ADBError::FileError(error_msg));
        }

        // 获取文件大小
        let file_size = fs::metadata(file_path)?.len() as usize;

        // 如果文件较小，直接使用标准推送
        if file_size <= chunk_size {
            debug!("文件大小较小，使用标准推送");
            return self.push(device_id, local_path, device_path, Some(options));
        }

        // 分块处理大文件
        let mut file = File::open(file_path).map_err(|e| {
            let error_msg = format!("无法打开文件 {}: {}", local_path, e);
            ADBError::FileError(error_msg)
        })?;

        info!(
            "将文件 {} 分成 {} 块传输",
            local_path,
            (file_size + chunk_size - 1) / chunk_size
        );

        // 创建设备上的临时目录
        let device_temp_dir = format!("{}.parts", device_path);
        self.shell(device_id, &format!("mkdir -p {}", device_temp_dir))?;

        // 分块传输
        let temp_dir = crate::utils::create_temp_dir_path("adb_push")?;

        let mut buffer = vec![0u8; chunk_size];
        let chunks_count = (file_size + chunk_size - 1) / chunk_size;

        // 创建单独的 TransferOptions 用于块传输，可能想要禁用某些选项
        let chunk_options = options.clone();

        // 对于部分传输可能不需要某些选项
        for i in 0..chunks_count {
            let part_file = temp_dir.join(format!("part{}", i));
            let bytes_read = file.read(&mut buffer[..]).map_err(|e| {
                let error_msg = format!("读取文件块失败: {}", e);
                ADBError::FileError(error_msg)
            })?;

            // 创建临时部分文件
            {
                let mut part = File::create(&part_file).map_err(|e| {
                    let error_msg = format!("创建临时文件失败: {}", e);
                    ADBError::FileError(error_msg)
                })?;

                part.write_all(&buffer[..bytes_read]).map_err(|e| {
                    let error_msg = format!("写入临时文件失败: {}", e);
                    ADBError::FileError(error_msg)
                })?;
            }

            // 推送此部分到设备
            let device_part_path = format!("{}/part{}", device_temp_dir, i);
            let push_result = self.push(
                device_id,
                part_file.to_str().unwrap(),
                &device_part_path,
                Some(chunk_options.clone()),
            );

            // 删除临时部分文件
            let _ = fs::remove_file(part_file);

            // 检查推送结果
            push_result.map_err(|e| {
                let error_msg = format!("推送文件块失败: {}", e);
                ADBError::CommandError(error_msg)
            })?;

            debug!("已推送块 {}/{}", i + 1, chunks_count);
        }

        // 合并所有部分
        let cat_cmd = format!(
            "cat {}/* > {} && rm -rf {}",
            device_temp_dir, device_path, device_temp_dir
        );
        self.shell(device_id, &cat_cmd)?;

        info!("已成功推送和合并大文件 {} 到 {}", local_path, device_path);

        // 清理临时目录
        let _ = fs::remove_dir_all(temp_dir);

        Ok(())
    }

    /// 文件存在性检查
    pub fn file_exists(&self, device_id: &str, path: &str) -> ADBResult<bool> {
        let result = self.shell(
            device_id,
            &format!("[ -e {} ] && echo 'exists' || echo 'not exists'", path),
        )?;
        Ok(result.trim() == "exists")
    }

    /// 获取文件/目录大小
    pub fn get_file_size(&self, device_id: &str, path: &str) -> ADBResult<u64> {
        // 检查是文件还是目录
        let is_dir = self
            .shell(
                device_id,
                &format!("[ -d {} ] && echo 'true' || echo 'false'", path),
            )?
            .trim()
            == "true";

        if is_dir {
            // 对于目录，使用 du 命令
            let output = self.shell(device_id, &format!("du -sk {} | cut -f1", path))?;
            let size_kb = output
                .trim()
                .parse::<u64>()
                .map_err(|_| ADBError::CommandError(format!("无法获取目录大小: {}", path)))?;

            Ok(size_kb * 1024) // 转换为字节
        } else {
            // 对于文件，使用 wc 命令
            let output = self.shell(device_id, &format!("wc -c < {}", path))?;
            let size = output
                .trim()
                .parse::<u64>()
                .map_err(|_| ADBError::CommandError(format!("无法获取文件大小: {}", path)))?;

            Ok(size)
        }
    }

    /// 创建目录
    pub fn create_directory(&self, device_id: &str, path: &str) -> ADBResult<()> {
        // 递归创建目录
        self.shell(device_id, &format!("mkdir -p {}", path))?;

        // 验证目录是否创建成功
        let exists = self.file_exists(device_id, path)?;
        if !exists {
            return Err(ADBError::CommandError(format!("无法创建目录: {}", path)));
        }

        Ok(())
    }

    /// 删除文件或目录
    pub fn remove_path(&self, device_id: &str, path: &str, recursive: bool) -> ADBResult<()> {
        // 检查路径是否存在
        let exists = self.file_exists(device_id, path)?;
        if !exists {
            return Err(ADBError::CommandError(format!("路径不存在: {}", path)));
        }

        // 检查是文件还是目录
        let is_dir = self
            .shell(
                device_id,
                &format!("[ -d {} ] && echo 'true' || echo 'false'", path),
            )?
            .trim()
            == "true";

        if is_dir {
            if recursive {
                // 递归删除目录
                self.shell(device_id, &format!("rm -rf {}", path))?;
            } else {
                // 删除空目录
                let output = self.shell(device_id, &format!("rmdir {}", path));

                // 检查是否因为目录非空而失败
                if let Err(e) = &output {
                    if let ADBError::DeviceError(msg) = e {
                        if msg.contains("Directory not empty") {
                            return Err(ADBError::CommandError(
                                "目录不为空，使用 recursive=true 递归删除".to_string(),
                            ));
                        }
                    }
                }

                output?;
            }
        } else {
            // 删除文件
            self.shell(device_id, &format!("rm {}", path))?;
        }

        // 验证路径是否已删除
        let still_exists = self.file_exists(device_id, path)?;
        if still_exists {
            return Err(ADBError::CommandError(format!("无法删除路径: {}", path)));
        }

        Ok(())
    }

    /// 复制设备上的文件
    pub fn copy_on_device(&self, device_id: &str, src_path: &str, dst_path: &str) -> ADBResult<()> {
        // 检查源文件是否存在
        if !self.file_exists(device_id, src_path)? {
            return Err(ADBError::FileError(format!("源文件不存在: {}", src_path)));
        }

        // 复制文件
        let command = format!("cp -f {} {}", src_path, dst_path);
        self.shell(device_id, &command)?;

        // 验证目标文件是否存在
        if !self.file_exists(device_id, dst_path)? {
            return Err(ADBError::CommandError(format!(
                "复制文件失败: {} -> {}",
                src_path, dst_path
            )));
        }

        Ok(())
    }

    /// 移动设备上的文件
    pub fn move_on_device(&self, device_id: &str, src_path: &str, dst_path: &str) -> ADBResult<()> {
        // 检查源文件是否存在
        if !self.file_exists(device_id, src_path)? {
            return Err(ADBError::FileError(format!("源文件不存在: {}", src_path)));
        }

        // 移动文件
        let command = format!("mv -f {} {}", src_path, dst_path);
        self.shell(device_id, &command)?;

        // 验证源文件不存在且目标文件存在
        if self.file_exists(device_id, src_path)? || !self.file_exists(device_id, dst_path)? {
            return Err(ADBError::CommandError(format!(
                "移动文件失败: {} -> {}",
                src_path, dst_path
            )));
        }

        Ok(())
    }

    /// 列出目录内容
    pub fn list_directory(&self, device_id: &str, path: &str) -> ADBResult<Vec<String>> {
        // 检查路径是否存在且是目录
        let exists = self.file_exists(device_id, path)?;
        let is_dir = self
            .shell(
                device_id,
                &format!("[ -d {} ] && echo 'true' || echo 'false'", path),
            )?
            .trim()
            == "true";

        if !exists || !is_dir {
            return Err(ADBError::FileError(format!(
                "路径不存在或不是目录: {}",
                path
            )));
        }

        // 列出目录内容
        let output = self.shell(device_id, &format!("ls -A {}", path))?;
        let files = output
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();

        Ok(files)
    }

    /// 获取文件最后修改时间
    pub fn get_file_mtime(&self, device_id: &str, path: &str) -> ADBResult<String> {
        // 检查文件是否存在
        if !self.file_exists(device_id, path)? {
            return Err(ADBError::FileError(format!("文件不存在: {}", path)));
        }

        // 获取文件最后修改时间
        let output = self.shell(device_id, &format!("stat -c %y {}", path))?;
        Ok(output.trim().to_string())
    }

    /// 检查设备上的可用空间
    pub fn get_available_space(&self, device_id: &str, path: &str) -> ADBResult<u64> {
        // 获取目标路径所在分区的可用空间
        let output = self.shell(device_id, &format!("df -k {} | tail -1", path))?;
        let parts: Vec<&str> = output.split_whitespace().collect();

        if parts.len() < 4 {
            return Err(ADBError::CommandError("无法解析可用空间信息".to_string()));
        }

        // 获取可用空间（KB）
        let available_kb = parts[3]
            .parse::<u64>()
            .map_err(|_| ADBError::CommandError("无法解析可用空间值".to_string()))?;

        // 转换为字节
        Ok(available_kb * 1024)
    }

    /// 计算文件或目录的 MD5 校验和
    pub fn compute_md5(&self, device_id: &str, path: &str) -> ADBResult<String> {
        // 检查文件是否存在
        if !self.file_exists(device_id, path)? {
            return Err(ADBError::FileError(format!("文件不存在: {}", path)));
        }

        // 检查是文件还是目录
        let is_dir = self
            .shell(
                device_id,
                &format!("[ -d {} ] && echo 'true' || echo 'false'", path),
            )?
            .trim()
            == "true";

        if is_dir {
            return Err(ADBError::CommandError("MD5 计算不支持目录".to_string()));
        }

        // 计算 MD5
        let output = self.shell(device_id, &format!("md5sum {}", path))?;
        let parts: Vec<&str> = output.split_whitespace().collect();

        if parts.is_empty() {
            return Err(ADBError::CommandError("无法计算 MD5".to_string()));
        }

        Ok(parts[0].to_string())
    }

    /// 写入文本到设备上的文件
    pub fn write_text_to_file(&self, device_id: &str, path: &str, content: &str) -> ADBResult<()> {
        // 创建父目录（如果需要）
        if let Some(parent_dir) = Path::new(path).parent().and_then(|p| p.to_str()) {
            if !parent_dir.is_empty() {
                let _ = self.create_directory(device_id, parent_dir);
            }
        }

        // 写入内容
        let escaped_content = content.replace("\"", "\\\"").replace("\n", "\\n");
        let command = format!("echo -e \"{}\" > {}", escaped_content, path);
        self.shell(device_id, &command)?;

        // 验证文件是否存在
        if !self.file_exists(device_id, path)? {
            return Err(ADBError::CommandError(format!("写入文件失败: {}", path)));
        }

        Ok(())
    }

    /// 读取设备上文件的文本内容
    pub fn read_text_from_file(&self, device_id: &str, path: &str) -> ADBResult<String> {
        // 检查文件是否存在
        if !self.file_exists(device_id, path)? {
            return Err(ADBError::FileError(format!("文件不存在: {}", path)));
        }

        // 读取文件内容
        let content = self.shell(device_id, &format!("cat {}", path))?;
        Ok(content)
    }

    /// 比较本地文件和设备文件是否相同
    pub fn compare_files(
        &self,
        device_id: &str,
        local_path: &str,
        device_path: &str,
    ) -> ADBResult<bool> {
        // 检查本地文件是否存在
        let local_file_path = Path::new(local_path);
        if !local_file_path.exists() || !local_file_path.is_file() {
            return Err(ADBError::FileError(format!(
                "本地文件不存在: {}",
                local_path
            )));
        }

        // 检查设备文件是否存在
        if !self.file_exists(device_id, device_path)? {
            return Err(ADBError::FileError(format!(
                "设备文件不存在: {}",
                device_path
            )));
        }

        // 计算本地文件的 MD5
        let local_md5 = match std::process::Command::new("md5sum")
            .arg(local_path)
            .output()
        {
            Ok(output) => {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    let parts: Vec<&str> = stdout.split_whitespace().collect();
                    if !parts.is_empty() {
                        parts[0].to_string()
                    } else {
                        return Err(ADBError::CommandError("无法计算本地文件 MD5".to_string()));
                    }
                } else {
                    return Err(ADBError::CommandError("计算本地文件 MD5 失败".to_string()));
                }
            }
            Err(e) => {
                return Err(ADBError::CommandError(format!(
                    "执行 md5sum 命令失败: {}",
                    e
                )))
            }
        };

        // 计算设备文件的 MD5
        let device_md5 = self.compute_md5(device_id, device_path)?;

        // 比较 MD5
        Ok(local_md5 == device_md5)
    }

    /// 同步目录 (本地到设备)
    pub fn sync_directory_to_device(
        &self,
        device_id: &str,
        local_dir: &str,
        device_dir: &str,
        exclude_patterns: Option<&[&str]>,
    ) -> ADBResult<()> {
        // 确保本地目录存在
        let local_dir_path = Path::new(local_dir);
        if !local_dir_path.exists() || !local_dir_path.is_dir() {
            return Err(ADBError::FileError(format!(
                "本地目录不存在: {}",
                local_dir
            )));
        }

        // 确保设备目录存在
        self.create_directory(device_id, device_dir)?;

        // 读取本地目录内容
        let entries = fs::read_dir(local_dir_path)
            .map_err(|e| ADBError::FileError(format!("无法读取本地目录: {}", e)))?;

        for entry in entries {
            let entry =
                entry.map_err(|e| ADBError::FileError(format!("读取目录条目失败: {}", e)))?;

            let file_name = entry.file_name();
            let file_name_str = file_name.to_string_lossy();

            // 检查排除模式
            if let Some(patterns) = exclude_patterns {
                let mut skip = false;
                for pattern in patterns {
                    if glob::Pattern::new(pattern).unwrap().matches(&file_name_str) {
                        skip = true;
                        break;
                    }
                }
                if skip {
                    continue;
                }
            }

            let local_path = entry.path();
            let device_path = format!("{}/{}", device_dir.trim_end_matches('/'), file_name_str);

            if local_path.is_dir() {
                // 递归同步子目录
                self.sync_directory_to_device(
                    device_id,
                    local_path.to_str().unwrap(),
                    &device_path,
                    exclude_patterns,
                )?;
            } else {
                // 推送文件
                self.push(device_id, local_path.to_str().unwrap(), &device_path, None)?;
            }
        }

        Ok(())
    }
}
