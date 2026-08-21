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

    #[response(raw)]
    pub struct RawResponse {
        pub code: i32,
        pub message: String,
    }

    impl Hello for HelloService {
        async fn hello() -> Result<HelloResponse, Box<dyn std::error::Error>> {
            Err("Just for test".into())
        }

        async fn login(_request: actix_web::web::Json<LoingRequest>) -> Result<LoingResponse, Box<dyn std::error::Error>> {
            Err("Just for test".into())
        }
    }

    #[actix_web::test]
    async fn test_response_wrapped_format() {
        use actix_web::Responder;

        let req = actix_web::test::TestRequest::default().to_http_request();
        let response = HelloResponse {
            message: "hello world",
        };

        let resp = response.respond_to(&req);

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let body_bytes = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["code"], 0);
        assert_eq!(json["data"]["message"], "hello world");
    }

    #[actix_web::test]
    async fn test_response_raw_format() {
        use actix_web::Responder;

        let req = actix_web::test::TestRequest::default().to_http_request();
        let response = RawResponse {
            code: 200,
            message: "success".to_string(),
        };

        let resp = response.respond_to(&req);

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let body_bytes = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        // Raw format should not be wrapped in {"code": 0, "data": ...}
        assert_eq!(json["code"], 200);
        assert_eq!(json["message"], "success");
        assert!(json.get("data").is_none());
    }

    #[actix_web::test]
    async fn test_response_login_response() {
        use actix_web::Responder;

        let req = actix_web::test::TestRequest::default().to_http_request();
        let response = LoingResponse {
            token: "test_token_123".to_string(),
        };

        let resp = response.respond_to(&req);

        assert_eq!(resp.status(), actix_web::http::StatusCode::OK);

        let body_bytes = actix_web::body::to_bytes(resp.into_body()).await.unwrap();
        let json: serde_json::Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["code"], 0);
        assert_eq!(json["data"]["token"], "test_token_123");
    }
}
