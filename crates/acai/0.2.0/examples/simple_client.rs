use acai::{
    JsonRpcRequest, JsonRpcResponse, Value,
    client::{Client, ClientConfig},
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client configuration
    let config = ClientConfig::new("http://127.0.0.1:3000");

    // Create client
    let client = Client::new(config)?;

    // Prepare parameters for the add method
    let params = vec![Value::from(5), Value::from(7)];

    // Create a request using the specialized constructor
    let request = JsonRpcRequest::new(serde_json::json!("request-1"), "add", params);

    // Send the request and await the response
    println!("Sending request to add 5 + 7");
    let response: JsonRpcResponse<Value> = client.send(request).await?;

    // Extract and print the result
    let result = response.result();
    println!("Response received: {:?}", result);

    Ok(())
}
