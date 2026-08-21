以下是为Cloudflare Token和ZeroSSL Token验证功能设计的补充方案，确保凭证安全有效：

### 验证模块设计
在现有结构中新增`auth`验证模块：
```diff
acme_commander/
├── src/
│   ├── auth/                  # 新增凭证验证模块
│   │   ├── mod.rs             # 验证器入口
│   │   ├── cloudflare.rs      # Cloudflare Token验证
│   │   └── zerossl.rs         # ZeroSSL API Key验证
│   └── ...
```

---

### Cloudflare Token验证 (`auth/cloudflare.rs`)
通过Cloudflare API验证Token权限并获取账户信息：
```rust
pub struct CloudflareAuth {
    token: String,  // Bearer Token
}

impl CloudflareAuth {
    /// 验证Token有效性并返回账户ID
    pub async fn verify(&self) -> Result<String, AuthError> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.cloudflare.com/client/v4/user/tokens/verify")
            .bearer_auth(&self.token)
            .send()
            .await?;

        let body: serde_json::Value = response.json().await?;
        
        // 检查API响应状态
        if body["success"].as_bool() != Some(true) {
            return Err(AuthError::InvalidToken(
                body["errors"][0]["message"].as_str().unwrap_or("Unknown").into()
            ));
        }
        
        // 验证DNS编辑权限
        let permissions = body["result"]["permissions"]
            .as_array()
            .ok_or(AuthError::MissingPermissions)?;
            
        if !permissions.iter().any(|p| 
            p["id"] == "dns_records:edit"
        ) {
            return Err(AuthError::InsufficientPermissions);
        }

        // 返回账户ID用于后续操作
        body["result"]["id"]
            .as_str()
            .map(|s| s.to_owned())
            .ok_or(AuthError::InvalidResponse)
    }
}
```

---

### ZeroSSL API Key验证 (`auth/zerossl.rs`)
验证ZeroSSL API Key并检查ACME访问权限：
```rust
pub struct ZeroSslAuth {
    api_key: String,  // API Key
}

impl ZeroSslAuth {
    /// 验证API Key有效性
    pub async fn verify(&self) -> Result<(), AuthError> {
        let client = reqwest::Client::new();
        let response = client
            .get("https://api.zerossl.com/acme/eab-credentials")
            .query(&[("access_key", &self.api_key)])
            .send()
            .await?;

        // 检查HTTP状态码
        if response.status() == 401 {
            return Err(AuthError::InvalidToken("Unauthorized".into()));
        }

        let body: serde_json::Value = response.json().await?;
        
        // 检查EAB凭证是否存在
        if body["success"] != true {
            return Err(AuthError::ServiceError(
                body["error"]["message"].as_str().unwrap_or("Unknown").into()
            ));
        }
        
        // 验证ACME功能是否启用
        if body["eab_credentials"].is_null() {
            return Err(AuthError::AcmeDisabled);
        }

        Ok(())
    }
}
```

---

### 统一验证接口 (`auth/mod.rs`)
```rust
pub enum Provider {
    Cloudflare,
    ZeroSsl,
}

pub async fn validate_credentials(
    provider: Provider,
    credential: &str
) -> Result<ValidationResult, AuthError> {
    match provider {
        Provider::Cloudflare => {
            let auth = CloudflareAuth::new(credential.to_owned());
            let account_id = auth.verify().await?;
            Ok(ValidationResult::Cloudflare { account_id })
        }
        Provider::ZeroSsl => {
            let auth = ZeroSslAuth::new(credential.to_owned());
            auth.verify().await?;
            Ok(ValidationResult::ZeroSsl)
        }
    }
}

// 验证结果类型
pub enum ValidationResult {
    Cloudflare { account_id: String },
    ZeroSsl,
}
```

---

### 错误处理 (`error.rs`)
```rust
#[derive(Debug, thiserror::Error)]
pub enum AuthError {
    #[error("Invalid token: {0}")]
    InvalidToken(String),
    
    #[error("Insufficient API permissions")]
    InsufficientPermissions,
    
    #[error("ACME not enabled for this account")]
    AcmeDisabled,
    
    #[error("Service error: {0}")]
    ServiceError(String),
    
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),
    
    #[error(transparent)]
    Json(#[from] serde_json::Error),
}
```

---

### 使用场景示例
在证书申请前进行凭证预验证：
```rust
// 验证Cloudflare Token
let cf_result = auth::validate_credentials(
    auth::Provider::Cloudflare, 
    &cloudflare_token
).await?;

// 验证ZeroSSL API Key
auth::validate_credentials(
    auth::Provider::ZeroSsl, 
    &zerossl_api_key
).await?;

// 使用验证通过的凭证进行后续操作
let account_id = match cf_result {
    ValidationResult::Cloudflare { account_id } => account_id,
    _ => unreachable!(),
};

// 创建DNS记录时使用已验证的账户ID
dns::cloudflare::create_txt_record(
    account_id,
    record_name,
    record_value
).await?;
```

---

### 安全增强措施
1. **凭证存储安全**：
   ```rust
   // 使用secrecy库保护内存中的凭证
   use secrecy::{Secret, ExposeSecret};
   
   struct SecureAuth {
       token: Secret<String>,
   }
   
   impl SecureAuth {
       fn verify(&self) {
           let exposed = self.token.expose_secret();
           // 使用暴露的字符串进行API调用
       }
   }
   ```

2. **请求限流保护**：
   ```rust
   // 使用governor库实现请求限流
   use governor::{Quota, RateLimiter};
   use std::num::NonZeroU32;
   
   static RATE_LIMITER: Lazy<RateLimiter> = Lazy::new(|| {
       RateLimiter::direct(Quota::per_minute(NonZeroU32::new(10).unwrap()))
   });
   
   async fn safe_verify(&self) {
       RATE_LIMITER.until_ready().await;
       // 执行验证请求
   }
   ```

3. **敏感日志过滤**：
   ```rust
   // 使用secrecy配合tracing过滤敏感信息
   #[derive(secrecy::DebugSecret)]
   struct SensitiveData {
       #[secret]
       token: String,
       safe_field: u32,
   }
   
   let data = SensitiveData { token: "xyz".into(), safe_field: 42 };
   info!(?data, "Debug不会暴露token"); // 日志中token自动替换为[REDACTED]
   ```

此设计确保在证书生成流程开始前验证凭证有效性，避免因无效凭证导致ACME流程中断，同时通过内存安全保护和请求限流增强系统安全性。