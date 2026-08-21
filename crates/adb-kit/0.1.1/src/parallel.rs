use crate::device::ADB;
use crate::error::{ADBError, ADBResult};
use crate::app::PackageInfo;
use log::{debug, warn};
use rayon::prelude::*;
use std::collections::HashMap;

impl ADB {
    /// 在多个设备上并行执行 shell 命令
    ///
    /// # 参数
    ///
    /// * `device_ids` - 设备 ID 列表
    /// * `command` - 要执行的 shell 命令
    ///
    /// # 返回值
    ///
    /// 返回一个 HashMap，键为设备 ID，值为命令执行结果
    pub fn parallel_shell(&self, device_ids: &[&str], command: &str) -> HashMap<String, ADBResult<String>> {
        device_ids
            .par_iter() // 使用 rayon 的并行迭代器
            .map(|&id| {
                (id.to_string(), self.shell(id, command))
            })
            .collect()
    }

    /// 在多个设备上并行安装应用
    ///
    /// # 参数
    ///
    /// * `device_ids` - 设备 ID 列表
    /// * `apk_path` - APK 文件路径
    ///
    /// # 返回值
    ///
    /// 返回一个 HashMap，键为设备 ID，值为安装结果
    pub fn parallel_install_app(&self, device_ids: &[&str], apk_path: &str) -> HashMap<String, ADBResult<()>> {
        device_ids
            .par_iter()
            .map(|&id| {
                (id.to_string(), self.install_app(id, apk_path))
            })
            .collect()
    }

    /// 在多个设备上并行卸载应用
    ///
    /// # 参数
    ///
    /// * `device_ids` - 设备 ID 列表
    /// * `package_name` - 包名
    ///
    /// # 返回值
    ///
    /// 返回一个 HashMap，键为设备 ID，值为卸载结果
    pub fn parallel_uninstall_app(&self, device_ids: &[&str], package_name: &str) -> HashMap<String, ADBResult<()>> {
        device_ids
            .par_iter()
            .map(|&id| {
                (id.to_string(), self.uninstall_app(id, package_name))
            })
            .collect()
    }

    /// 在多个设备上并行启动应用
    ///
    /// # 参数
    ///
    /// * `device_ids` - 设备 ID 列表
    /// * `package_name` - 包名
    /// * `activity` - 可选的 Activity 名称
    ///
    /// # 返回值
    ///
    /// 返回一个 HashMap，键为设备 ID，值为启动结果
    pub fn parallel_start_app(
        &self,
        device_ids: &[&str],
        package_name: &str,
        activity: Option<&str>,
    ) -> HashMap<String, ADBResult<bool>> {
        device_ids
            .par_iter()
            .map(|&id| {
                (id.to_string(), self.start_app(id, package_name, activity))
            })
            .collect()
    }

    /// 在多个设备上并行停止应用
    ///
    /// # 参数
    ///
    /// * `device_ids` - 设备 ID 列表
    /// * `package_name` - 包名
    ///
    /// # 返回值
    ///
    /// 返回一个 HashMap，键为设备 ID，值为停止结果
    pub fn parallel_stop_app(&self, device_ids: &[&str], package_name: &str) -> HashMap<String, ADBResult<()>> {
        device_ids
            .par_iter()
            .map(|&id| {
                (id.to_string(), self.stop_app(id, package_name))
            })
            .collect()
    }

    /// 在多个设备上并行获取包信息
    ///
    /// # 参数
    ///
    /// * `device_ids` - 设备 ID 列表
    /// * `package_name` - 包名
    ///
    /// # 返回值
    ///
    /// 返回一个 HashMap，键为设备 ID，值为包信息
    pub fn parallel_get_package_info(
        &self,
        device_ids: &[&str],
        package_name: &str,
    ) -> HashMap<String, ADBResult<PackageInfo>> {
        device_ids
            .par_iter()
            .map(|&id| {
                (id.to_string(), self.get_package_info_enhanced(id, package_name))
            })
            .collect()
    }

    /// 在多个设备上并行执行推送文件操作
    ///
    /// # 参数
    ///
    /// * `device_ids` - 设备 ID 列表
    /// * `local_path` - 本地文件路径
    /// * `device_path` - 设备上的目标路径
    ///
    /// # 返回值
    ///
    /// 返回一个 HashMap，键为设备 ID，值为推送结果
    pub fn parallel_push(
        &self,
        device_ids: &[&str],
        local_path: &str,
        device_path: &str,
    ) -> HashMap<String, ADBResult<()>> {
        device_ids
            .par_iter()
            .map(|&id| {
                (id.to_string(), self.push(id, local_path, device_path, None))
            })
            .collect()
    }

    /// 在多个设备上并行执行拉取文件操作
    ///
    /// # 参数
    ///
    /// * `operations` - 设备 ID 和文件路径的组合列表，每项包含(设备 ID, 设备文件路径, 本地目标路径)
    ///
    /// # 返回值
    ///
    /// 返回一个 HashMap，键为设备 ID，值为拉取结果
    pub fn parallel_pull(
        &self,
        operations: &[(String, String, String)],
    ) -> HashMap<String, ADBResult<()>> {
        operations
            .par_iter()
            .map(|(device_id, device_path, local_path)| {
                (device_id.clone(), self.pull(device_id, device_path, local_path, None))
            })
            .collect()
    }

    /// 检查多个设备是否在线
    ///
    /// # 参数
    ///
    /// * `device_ids` - 设备 ID 列表
    ///
    /// # 返回值
    ///
    /// 返回在线设备的列表
    pub fn filter_online_devices(&self, device_ids: &[&str]) -> ADBResult<Vec<String>> {
        let results = device_ids
            .par_iter()
            .map(|&id| {
                (id.to_string(), self.is_device_online(id))
            })
            .collect::<HashMap<String, ADBResult<bool>>>();

        let mut online_devices = Vec::new();
        for (id, result) in results {
            match result {
                Ok(true) => online_devices.push(id),
                Ok(false) => debug!("设备 {} 不在线", id),
                Err(e) => warn!("检查设备 {} 状态时出错: {}", id, e),
            }
        }

        Ok(online_devices)
    }

    /// 在所有在线设备上执行操作
    ///
    /// # 参数
    ///
    /// * `operation` - 要执行的操作闭包
    ///
    /// # 返回值
    ///
    /// 返回在线设备的操作结果
    pub fn on_all_online_devices<F, T>(&self, operation: F) -> ADBResult<HashMap<String, ADBResult<T>>>
    where
        F: Fn(&str) -> ADBResult<T> + Send + Sync,
        T: Send,
    {
        // 获取所有设备
        let devices = self.list_devices()?;

        // 筛选在线设备
        let online_devices: Vec<String> = devices
            .iter()
            .filter(|d| d.is_online())
            .map(|d| d.id.clone())
            .collect();

        if online_devices.is_empty() {
            return Err(ADBError::DeviceError("没有在线设备".to_string()));
        }

        // 并行执行操作
        let results = online_devices
            .par_iter()
            .map(|id| {
                (id.clone(), operation(id))
            })
            .collect();

        Ok(results)
    }

    /// 在所有指定设备上并行执行多个命令
    pub fn parallel_commands(
        &self,
        device_ids: &[&str],
        commands: &[&str],
    ) -> HashMap<String, Vec<ADBResult<String>>> {
        device_ids
            .par_iter()
            .map(|&id| {
                let results = commands
                    .iter()
                    .map(|&cmd| self.shell(id, cmd))
                    .collect();

                (id.to_string(), results)
            })
            .collect()
    }

    /// 在所有在线设备上启动同一应用
    pub fn start_app_on_all_devices(
        &self,
        package_name: &str,
        activity: Option<&str>,
    ) -> ADBResult<HashMap<String, ADBResult<bool>>> {
        self.on_all_online_devices(|device_id| {
            self.start_app(device_id, package_name, activity)
        })
    }

    /// 在所有在线设备上停止同一应用
    pub fn stop_app_on_all_devices(
        &self,
        package_name: &str,
    ) -> ADBResult<HashMap<String, ADBResult<()>>> {
        self.on_all_online_devices(|device_id| {
            self.stop_app(device_id, package_name)
        })
    }
}