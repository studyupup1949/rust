use ada_sdk::AdaClient;
use ada_sdk::proto::IngestRequest;

async fn invalid(client: AdaClient) {
    client.ingest(IngestRequest::default()).await;
}

fn main() {}
