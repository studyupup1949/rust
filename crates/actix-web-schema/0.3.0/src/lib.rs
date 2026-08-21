pub use actix_web_schema_macro::*;

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试服务
    #[service]
    pub trait Hello {
        /// 一个测试接口
        #[get("/api/v1/hello")]
        async fn hello() -> Result<HelloResponse, Box<dyn std::error::Error>>;

        /// 登录接口
        #[post("/api/v1/login")]
        async fn login(request: actix_web::web::Json<LoingRequest>) -> Result<LoingResponse, Box<dyn std::error::Error>>;
    }

    /// Hello的响应
    #[response]
    pub struct HelloResponse {
        pub message: &'static str,
    }

    /// 登录接口的请求
    #[request]
    pub struct LoingRequest {
        pub username: String,
        pub password: String,
    }

    #[response]
    pub struct LoingResponse {
        pub token: String,
    }

    impl Hello for HelloService {
        async fn hello() -> Result<HelloResponse, Box<dyn std::error::Error>> {
            Err("Just for test".into())
        }

        async fn login(_request: actix_web::web::Json<LoingRequest>) -> Result<LoingResponse, Box<dyn std::error::Error>> {
            Err("Just for test".into())
        }
    }
}
