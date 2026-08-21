//! 对象存储配置模块
//!
//! 提供多种云对象存储服务的配置，包括：
//! - 腾讯云 COS
//! - 阿里云 OSS
//! - AWS S3
//! - MinIO
//! - 华为云 OBS
//! - RustFS（本地文件系统）

use serde::{Deserialize, Serialize};

/// 对象存储配置
///
/// 支持多种云存储服务商和本地文件系统
///
/// # 字段说明
///
/// - `provider`: 存储提供商，可选值：tencent/aliyun/aws/minio/huawei/rustfs
/// - `tencent`: 腾讯云 COS 配置
/// - `aliyun`: 阿里云 OSS 配置
/// - `aws`: AWS S3 配置
/// - `minio`: MinIO 配置
/// - `huawei`: 华为云 OBS 配置
/// - `rustfs`: 本地文件系统配置
///
/// # 示例
///
/// ```rust
/// use admin_config::CosConfig;
///
/// let config = CosConfig {
///     provider: "tencent".to_string(),
///     ..Default::default()
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CosConfig {
    /// 对象存储提供商 (tencent/aliyun/aws/minio/huawei/rustfs)
    pub provider: String,
    /// 腾讯云 COS 配置
    #[serde(default)]
    pub tencent: TencentCosConfig,
    /// 阿里云 OSS 配置
    #[serde(default)]
    pub aliyun: AliyunOssConfig,
    /// AWS S3 配置
    #[serde(default)]
    pub aws: AwsS3Config,
    /// MinIO 配置
    #[serde(default)]
    pub minio: MinioConfig,
    /// 华为云 OBS 配置
    #[serde(default)]
    pub huawei: HuaweiObsConfig,
    /// RustFS 本地文件系统配置
    #[serde(default)]
    pub rustfs: RustFsConfig,
}

/// 腾讯云 COS 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TencentCosConfig {
    /// Secret ID
    pub secret_id: String,
    /// Secret Key
    pub secret_key: String,
    /// 存储桶名称
    pub bucket: String,
    /// 区域
    pub region: String,
    /// CDN 域名
    pub cdn_domain: Option<String>,
}

/// 阿里云 OSS 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AliyunOssConfig {
    /// Access Key ID
    pub access_key_id: String,
    /// Access Key Secret
    pub access_key_secret: String,
    /// 存储桶名称
    pub bucket: String,
    /// 区域节点 (例如: oss-cn-hangzhou)
    pub endpoint: String,
    /// CDN 域名
    pub cdn_domain: Option<String>,
}

/// AWS S3 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AwsS3Config {
    /// Access Key ID
    pub access_key_id: String,
    /// Secret Access Key
    pub secret_access_key: String,
    /// 存储桶名称
    pub bucket: String,
    /// 区域 (例如: us-east-1)
    pub region: String,
    /// 自定义 Endpoint (可选，用于兼容 S3 的服务如 MinIO)
    pub endpoint: Option<String>,
    /// CDN 域名
    pub cdn_domain: Option<String>,
}

/// MinIO 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MinioConfig {
    /// Access Key
    pub access_key: String,
    /// Secret Key
    pub secret_key: String,
    /// 存储桶名称
    pub bucket: String,
    /// Endpoint (例如: http://localhost:9000)
    pub endpoint: String,
    /// 是否使用 SSL
    pub use_ssl: bool,
}

/// 华为云 OBS 配置
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HuaweiObsConfig {
    /// Access Key ID
    pub access_key_id: String,
    /// Secret Access Key
    pub secret_access_key: String,
    /// 存储桶名称
    pub bucket: String,
    /// 区域节点 (例如: cn-north-4)
    pub endpoint: String,
    /// CDN 域名
    pub cdn_domain: Option<String>,
}

/// RustFS 本地文件系统配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RustFsConfig {
    /// 本地存储根目录路径
    pub root_path: String,
    /// 公共访问 URL 前缀
    pub public_url_prefix: String,
    /// 是否自动创建目录
    pub auto_create_dir: bool,
}

impl Default for RustFsConfig {
    fn default() -> Self {
        Self {
            root_path: "./uploads".to_string(),
            public_url_prefix: "/uploads".to_string(),
            auto_create_dir: true,
        }
    }
}

impl CosConfig {
    /// 获取当前提供商的存储桶名称
    pub fn get_bucket(&self) -> String {
        match self.provider.as_str() {
            "tencent" => self.tencent.bucket.clone(),
            "aliyun" => self.aliyun.bucket.clone(),
            "aws" => self.aws.bucket.clone(),
            "minio" => self.minio.bucket.clone(),
            "huawei" => self.huawei.bucket.clone(),
            "rustfs" => String::new(),
            _ => String::new(),
        }
    }

    /// 获取当前提供商的 CDN 域名
    pub fn get_cdn_domain(&self) -> Option<String> {
        match self.provider.as_str() {
            "tencent" => self.tencent.cdn_domain.clone(),
            "aliyun" => self.aliyun.cdn_domain.clone(),
            "aws" => self.aws.cdn_domain.clone(),
            "huawei" => self.huawei.cdn_domain.clone(),
            _ => None,
        }
    }

    /// 获取当前提供商的 Endpoint
    pub fn get_endpoint(&self) -> Option<String> {
        match self.provider.as_str() {
            "minio" => Some(self.minio.endpoint.clone()),
            "huawei" => Some(self.huawei.endpoint.clone()),
            "aliyun" => Some(self.aliyun.endpoint.clone()),
            "aws" => self.aws.endpoint.clone(),
            _ => None,
        }
    }

    /// 获取本地文件系统的根路径（仅 RustFS）
    pub fn get_root_path(&self) -> Option<String> {
        match self.provider.as_str() {
            "rustfs" => Some(self.rustfs.root_path.clone()),
            _ => None,
        }
    }
}
