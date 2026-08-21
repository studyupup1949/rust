use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use std::sync::{Arc, Mutex};

use crate::config::ADBConfig;

/// ADB 设备状态枚举
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceStatus {
    Online,
    Offline,
    Unauthorized,
    Recovery,
    Sideload,
    Bootloader,
    Other(String),
}

impl fmt::Display for DeviceStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DeviceStatus::Online => write!(f, "online"),
            DeviceStatus::Offline => write!(f, "offline"),
            DeviceStatus::Unauthorized => write!(f, "unauthorized"),
            DeviceStatus::Recovery => write!(f, "recovery"),
            DeviceStatus::Sideload => write!(f, "sideload"),
            DeviceStatus::Bootloader => write!(f, "bootloader"),
            DeviceStatus::Other(s) => write!(f, "{}", s),
        }
    }
}

impl From<&str> for DeviceStatus {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "device" | "online" => DeviceStatus::Online,
            "offline" => DeviceStatus::Offline,
            "unauthorized" => DeviceStatus::Unauthorized,
            "recovery" => DeviceStatus::Recovery,
            "sideload" => DeviceStatus::Sideload,
            "bootloader" | "fastboot" => DeviceStatus::Bootloader,
            _ => DeviceStatus::Other(s.to_string()),
        }
    }
}

/// ADB 设备结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ADBDevice {
    pub id: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub model: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub product: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport_id: Option<String>,
    pub status: DeviceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub properties: Option<HashMap<String, String>>,
}

impl ADBDevice {
    /// 创建新设备实例
    pub fn new(id: &str, status: impl Into<DeviceStatus>) -> Self {
        Self {
            id: id.to_string(),
            name: format!("Device {}", id),
            model: None,
            product: None,
            transport_id: None,
            status: status.into(),
            properties: None,
        }
    }

    /// 检查设备是否在线
    pub fn is_online(&self) -> bool {
        self.status == DeviceStatus::Online
    }

    /// 设置设备名称
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = name.to_string();
        self
    }

    /// 设置设备模型
    pub fn with_model(mut self, model: &str) -> Self {
        self.model = Some(model.to_string());
        self
    }

    /// 设置设备产品信息
    pub fn with_product(mut self, product: &str) -> Self {
        self.product = Some(product.to_string());
        self
    }

    /// 设置传输 ID
    pub fn with_transport_id(mut self, transport_id: &str) -> Self {
        self.transport_id = Some(transport_id.to_string());
        self
    }

    /// 添加设备属性
    pub fn add_property(mut self, key: &str, value: &str) -> Self {
        if self.properties.is_none() {
            self.properties = Some(HashMap::new());
        }

        if let Some(props) = &mut self.properties {
            props.insert(key.to_string(), value.to_string());
        }

        self
    }
}

/// ADB 连接池类型
type DevicePool = HashMap<String, Arc<Mutex<std::process::Child>>>;

/// ADB 主结构体
#[derive(Clone, Debug)]
pub struct ADB {
    pub config: ADBConfig,
    pub(crate) connections: Arc<Mutex<DevicePool>>,
}

impl ADB {
    /// 创建新的 ADB 实例
    pub fn new(config: Option<ADBConfig>) -> Self {
        Self {
            config: config.unwrap_or_default(),
            connections: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 获取 ADB 路径
    pub fn adb_path(&self) -> &std::path::PathBuf {
        &self.config.path
    }
}