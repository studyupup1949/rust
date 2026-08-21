use std::sync::Arc;

use acai::{
    JsonRpcError, Value,
    server::{self, MethodRouter, Server, ServerConfig},
};

// A simple handler that adds two numbers
async fn add_handler(_: Arc<()>, params: Vec<Value>) -> Result<Value, JsonRpcError> {
    if params.len() != 2 {
        return Err(JsonRpcError::invalid_parameters(
            "expected exactly 2 numbers",
        ));
    }

    let a = params[0]
        .as_i64()
        .ok_or_else(|| JsonRpcError::invalid_parameters("first parameter must be an integer"))?;
    let b = params[1]
        .as_i64()
        .ok_or_else(|| JsonRpcError::invalid_parameters("second parameter must be an integer"))?;

    let result = a + b;
    Ok(Value::from(result))
}

#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    // Create a router and register our add method
    let mut router = MethodRouter::new();

    // Create the handler using make_typed_handler
    let typed_handler = server::make_typed_handler(Arc::new(()), add_handler);

    router.register("add", typed_handler);

    // Create server configuration
    let config = ServerConfig::new("127.0.0.1:3000")?;

    // Create and run the server
    let server = Server::new(config, Arc::new(router));
    println!("A2A server listening on http://127.0.0.1:3000");
    server.serve().await?;

    Ok(())
}
