//! # JSON-RPC Client Module
//!
//! This module provides the infrastructure for making JSON-RPC calls. It includes:
//! - The [`JsonRpcClient`] trait for sending requests.
//! - Extensive implementations for Rust primitives, tuples, and collections.
//! - The [`rpc_method`] macro for high-level, declarative API definitions.
//!
//! The library uses a "parameter-first" approach where you can call RPC methods
//! directly on the data you wish to send as parameters.

use crate::{JsonRpcId, error::RpcError, request::JsonRpcRequest, response::JsonRpcResponse};
use serde::Serialize;

pub use a_rs_jsonrpc_macros::JsonRpcClient;
pub use a_rs_jsonrpc_macros::rpc_method;

/// The core trait for sending JSON-RPC requests.
///
/// This trait is implemented for a wide variety of types (scalars, tuples, vectors, etc.),
/// allowing them to be used as the `params` field in a JSON-RPC call.
///
/// ### Example: Sending a request from a tuple
/// ```rust
/// let params = (10, "hello");
/// let response = params.send_v2_request::<String>(url, "application/json", "myMethod").await?;
/// ```

#[async_trait::async_trait]
pub trait JsonRpcClient {
    /// Sends a JSON-RPC 1.0 request using `self` as the parameters (serialized as an array).
    async fn send_v1_request<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned;

    /// Sends a JSON-RPC 2.0 request using `self` as the parameters (serialized as an array).
    async fn send_v2_request<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned;

    /// Sends a JSON-RPC 1.0 request using `self` as the parameters (serialized as an object).
    /// Defaults to array-style if not overridden.
    async fn send_v1_request_obj<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        self.send_v1_request(url, content_type, method).await
    }

    /// Sends a JSON-RPC 2.0 request using `self` as the parameters (serialized as an object).
    /// Defaults to array-style if not overridden.
    async fn send_v2_request_obj<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        self.send_v2_request(url, content_type, method).await
    }
}

/// A helper trait for executing RPC calls.
/// Typically used by internal macro expansions to simplify the call site.
#[async_trait::async_trait]
pub trait JsonRpcClientCall {
    async fn call_rpc_v1<R>(&self) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned;

    async fn call_rpc_v2<R>(&self) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned;

    async fn call_rpc_v1_obj<R>(&self) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned;

    async fn call_rpc_v2_obj<R>(&self) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned;
}

// Internal implementations for:
// - Scalars: i8-i64, u8-u64, f32/f64, bool (Serialized as single-element arrays)
// - Tuples: (T1, T2, ...) up to 16 elements (Serialized as arrays)
// - Collections: Vec<T>, &[T] (Serialized as arrays)
// - Strings: String, &str (Serialized as single-element arrays)
// - Options: Some(T) or None
// - Unit type: () (Serialized as empty parameters)

macro_rules! impl_scalar_jsonrpc_client {
    ($rec:ty) => {
        #[async_trait::async_trait]
        impl JsonRpcClient for $rec where $rec: Serialize {
            async fn send_v1_request<R>(
                &self,
                url: &str,
                content_type: &str,
                method: &str,
            ) -> Result<JsonRpcResponse<R>, RpcError>
            where
                R: serde::de::DeserializeOwned,
            {
                let id = JsonRpcId::next_number();
                let mut body: JsonRpcRequest<Vec<serde_json::Value>> =
                    JsonRpcRequest::new_v1(id, method);
                body.add_param(*self);
                tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
                let resp = reqwest::Client::new()
                    .post(url)
                    .header("Content-Type", content_type)
                    .json(&body)
                    .send()
                    .await?;
                let text = resp.text().await?;
                tracing::debug!("jsonrpc response body: {}", text);
                Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
            }

            async fn send_v2_request<R>(
                &self,
                url: &str,
                content_type: &str,
                method: &str,
            ) -> Result<JsonRpcResponse<R>, RpcError>
            where
                R: serde::de::DeserializeOwned,
            {
                let id = JsonRpcId::next_number();
                let mut body: JsonRpcRequest<Vec<serde_json::Value>> =
                    JsonRpcRequest::new_v2(id, method);
                body.add_param(*self);
                tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
                let resp = reqwest::Client::new()
                    .post(url)
                    .header("Content-Type", content_type)
                    .json(&body)
                    .send()
                    .await
                    .unwrap();
                let text = resp.text().await?;
                tracing::debug!("jsonrpc response body: {}", text);
                Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
            }
        }
    };
    ($($rec:ty),*) => {
        $(impl_scalar_jsonrpc_client!($rec);)*
    };
}

impl_scalar_jsonrpc_client!(
    i8, u8, i16, u16, i32, u32, i64, u64, isize, usize, f32, f64, bool
);

macro_rules! impl_tuple_jsonrpc_client {
    ($($ty:ident),*) => {
        #[async_trait::async_trait]
        #[allow(non_snake_case)]
        impl<$($ty),*> JsonRpcClient for ($($ty,)*)
        where
            $(
                serde_json::Value: From<$ty>,
                $ty: Clone + Serialize + Send + Sync,
            )*
        {
            async fn send_v1_request<R>(
                &self,
                url: &str,
                content_type: &str,
                method: &str,
            ) -> Result<JsonRpcResponse<R>, RpcError>
            where
                R: serde::de::DeserializeOwned,
            {
                let id = JsonRpcId::next_number();
                let mut body: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest::new_v1(id, method);

                let ($($ty,)*) = self;
                $(
                    body.add_param($ty.clone());
                )*
                tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));

                let resp = reqwest::Client::new()
                    .post(url)
                    .header("Content-Type", content_type)
                    .json(&body)
                    .send()
                    .await?;
                let text = resp.text().await?;
                tracing::debug!("jsonrpc response body: {}", text);
                Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
            }

            async fn send_v2_request<R>(
                &self,
                url: &str,
                content_type: &str,
                method: &str,
            ) -> Result<JsonRpcResponse<R>, RpcError>
            where
                R: serde::de::DeserializeOwned,
            {
                let id = JsonRpcId::next_number();
                let mut body: JsonRpcRequest<Vec<serde_json::Value>> =
                    JsonRpcRequest::new_v2(id, method);

                let ($($ty,)*) = self;
                $(
                    body.add_param($ty.clone());
                )*
                tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));

                let resp = reqwest::Client::new()
                    .post(url)
                    .header("Content-Type", content_type)
                    .json(&body)
                    .send()
                    .await?;
                let text = resp.text().await?;
                tracing::debug!("jsonrpc response body: {}", text);
                Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
            }
        }
    };
}

macro_rules! generate_tuple_impls {
    ($first:ident) => {
        impl_tuple_jsonrpc_client!($first);
    };
    ($first:ident, $($rest:ident),*) => {
        impl_tuple_jsonrpc_client!($first, $($rest),*);
        generate_tuple_impls!($($rest),*);
    };
}

generate_tuple_impls!(
    T15, T14, T13, T12, T11, T10, T9, T8, T7, T6, T5, T4, T3, T2, T1, T0
);

#[async_trait::async_trait]
impl<T> JsonRpcClient for Vec<T>
where
    T: Clone + Serialize + Send + Sync,
    serde_json::Value: From<T>,
{
    async fn send_v1_request<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = JsonRpcId::next_number();
        let mut body: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest::new_v1(id, method);
        for item in self {
            body.add_param(item.clone());
        }
        tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
        let resp = reqwest::Client::new()
            .post(url)
            .header("Content-Type", content_type)
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        tracing::debug!("jsonrpc response body: {}", text);
        Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
    }

    async fn send_v2_request<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = JsonRpcId::next_number();
        let mut body: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest::new_v2(id, method);
        for item in self {
            body.add_param(item.clone());
        }
        tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
        let resp = reqwest::Client::new()
            .post(url)
            .header("Content-Type", content_type)
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        tracing::debug!("jsonrpc response body: {}", text);
        Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
    }
}

#[async_trait::async_trait]
impl<T> JsonRpcClient for &[T]
where
    T: Clone + Serialize + Send + Sync,
    serde_json::Value: From<T>,
{
    async fn send_v1_request<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = JsonRpcId::next_number();
        let mut body: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest::new_v1(id, method);
        for item in *self {
            body.add_param(item.clone());
        }
        tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
        let resp = reqwest::Client::new()
            .post(url)
            .header("Content-Type", content_type)
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        tracing::debug!("jsonrpc response body: {}", text);
        Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
    }

    async fn send_v2_request<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = JsonRpcId::next_number();
        let mut body: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest::new_v2(id, method);
        for item in *self {
            body.add_param(item.clone());
        }
        tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
        let resp = reqwest::Client::new()
            .post(url)
            .header("Content-Type", content_type)
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        tracing::debug!("jsonrpc response body: {}", text);
        Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
    }
}

#[async_trait::async_trait]
impl JsonRpcClient for () {
    async fn send_v1_request<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = JsonRpcId::next_number();
        let mut body: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest::new_v1(id, method);
        body.set_params(vec![]);
        tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
        let resp = reqwest::Client::new()
            .post(url)
            .header("Content-Type", content_type)
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        tracing::debug!("jsonrpc response body: {}", text);
        Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
    }

    async fn send_v2_request<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = JsonRpcId::next_number();
        let mut body: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest::new_v2(id, method);
        body.set_params(vec![]);
        tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
        let resp = reqwest::Client::new()
            .post(url)
            .header("Content-Type", content_type)
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        tracing::debug!("jsonrpc response body: {}", text);
        Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
    }

    async fn send_v1_request_obj<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = JsonRpcId::next_number();
        let mut body: JsonRpcRequest<serde_json::Value> = JsonRpcRequest::new_v1(id, method);
        body.set_params(serde_json::json!({}));
        tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
        let resp = reqwest::Client::new()
            .post(url)
            .header("Content-Type", content_type)
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        tracing::debug!("jsonrpc response body: {}", text);
        Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
    }

    async fn send_v2_request_obj<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = JsonRpcId::next_number();
        let mut body: JsonRpcRequest<serde_json::Value> = JsonRpcRequest::new_v2(id, method);
        body.set_params(serde_json::json!({}));
        tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
        let resp = reqwest::Client::new()
            .post(url)
            .header("Content-Type", content_type)
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        tracing::debug!("jsonrpc response body: {}", text);
        Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
    }
}

#[async_trait::async_trait]
impl<T> JsonRpcClient for Option<T>
where
    T: JsonRpcClient + Serialize + Send + Sync,
    serde_json::Value: From<T>,
{
    async fn send_v1_request<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = JsonRpcId::next_number();
        let mut body: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest::new_v1(id, method);
        if let Some(inner) = self {
            return inner.send_v1_request(url, content_type, method).await;
        } else {
            body.set_params(vec![]);
        }
        tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
        let resp = reqwest::Client::new()
            .post(url)
            .header("Content-Type", content_type)
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        tracing::debug!("jsonrpc response body: {}", text);
        Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
    }

    async fn send_v2_request<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = JsonRpcId::next_number();
        let mut body: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest::new_v2(id, method);
        if let Some(inner) = self {
            return inner.send_v2_request(url, content_type, method).await;
        } else {
            body.set_params(vec![]);
        }
        tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
        let resp = reqwest::Client::new()
            .post(url)
            .header("Content-Type", content_type)
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        tracing::debug!("jsonrpc response body: {}", text);
        Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
    }
}

#[async_trait::async_trait]
impl JsonRpcClient for String {
    async fn send_v1_request<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = JsonRpcId::next_number();
        let mut body: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest::new_v1(id, method);
        body.add_param(self.as_str());
        tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
        let resp = reqwest::Client::new()
            .post(url)
            .header("Content-Type", content_type)
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        tracing::debug!("jsonrpc response body: {}", text);
        Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
    }

    async fn send_v2_request<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = JsonRpcId::next_number();
        let mut body: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest::new_v2(id, method);
        body.add_param(self.as_str());
        tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
        let resp = reqwest::Client::new()
            .post(url)
            .header("Content-Type", content_type)
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        tracing::debug!("jsonrpc response body: {}", text);
        Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
    }
}

#[async_trait::async_trait]
impl JsonRpcClient for &str {
    async fn send_v1_request<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = JsonRpcId::next_number();
        let mut body: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest::new_v1(id, method);
        body.add_param(*self);
        tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
        let resp = reqwest::Client::new()
            .post(url)
            .header("Content-Type", content_type)
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        tracing::debug!("jsonrpc response body: {}", text);
        Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
    }

    async fn send_v2_request<R>(
        &self,
        url: &str,
        content_type: &str,
        method: &str,
    ) -> Result<JsonRpcResponse<R>, RpcError>
    where
        R: serde::de::DeserializeOwned,
    {
        let id = JsonRpcId::next_number();
        let mut body: JsonRpcRequest<Vec<serde_json::Value>> = JsonRpcRequest::new_v2(id, method);
        body.add_param(*self);
        tracing::debug!("jsonrpc request body: {:?}", serde_json::to_string(&body));
        let resp = reqwest::Client::new()
            .post(url)
            .header("Content-Type", content_type)
            .json(&body)
            .send()
            .await?;
        let text = resp.text().await?;
        tracing::debug!("jsonrpc response body: {}", text);
        Ok(serde_json::from_str::<JsonRpcResponse<R>>(&text)?)
    }
}
