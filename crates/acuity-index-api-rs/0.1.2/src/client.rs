use crate::error::{IndexerApiError, ServerError, SubscriptionTerminated};
use crate::types::{
    EmptyPayload, Envelope, ErrorPayload, EventNotification, EventNotificationPayload, EventsResponse,
    GetEventsPayload, Key, PalletMeta, RequestMessage, Span, StatusUpdate,
    SubscribeEventsPayload, SubscriptionStatusPayload,
    SubscriptionTerminatedPayload,
};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};
use tokio::sync::{Mutex, mpsc, oneshot};
use tokio_tungstenite::{connect_async, tungstenite::Message};

type PendingSender = oneshot::Sender<Result<Envelope, IndexerApiError>>;
type StatusSubscribers = Arc<Mutex<HashMap<u64, mpsc::Sender<Result<StatusUpdate, IndexerApiError>>>>>;
type EventSubscribers = Arc<Mutex<HashMap<u64, EventSubscriber>>>;

#[derive(Clone)]
struct EventSubscriber {
    key: Key,
    sender: mpsc::Sender<Result<EventNotification, IndexerApiError>>,
}

#[derive(Clone)]
pub struct IndexerClient {
    writer: Arc<Mutex<futures::stream::SplitSink<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>, Message>>>,
    pending: Arc<Mutex<HashMap<u64, PendingSender>>>,
    status_subscribers: StatusSubscribers,
    event_subscribers: EventSubscribers,
    next_id: Arc<AtomicU64>,
}

pub struct StatusSubscription {
    client: IndexerClient,
    id: u64,
    receiver: mpsc::Receiver<Result<StatusUpdate, IndexerApiError>>,
}

pub struct EventSubscription {
    client: IndexerClient,
    id: u64,
    key: Key,
    receiver: mpsc::Receiver<Result<EventNotification, IndexerApiError>>,
}

impl IndexerClient {
    pub async fn connect(url: &str) -> Result<Self, IndexerApiError> {
        let (stream, _) = connect_async(url).await?;
        let (writer, reader) = stream.split();

        let client = Self {
            writer: Arc::new(Mutex::new(writer)),
            pending: Arc::new(Mutex::new(HashMap::new())),
            status_subscribers: Arc::new(Mutex::new(HashMap::new())),
            event_subscribers: Arc::new(Mutex::new(HashMap::new())),
            next_id: Arc::new(AtomicU64::new(1)),
        };

        tokio::spawn(run_reader(
            reader,
            Arc::clone(&client.pending),
            Arc::clone(&client.status_subscribers),
            Arc::clone(&client.event_subscribers),
        ));

        Ok(client)
    }

    pub async fn close(&self) -> Result<(), IndexerApiError> {
        self.writer.lock().await.close().await?;
        Ok(())
    }

    pub async fn status(&self) -> Result<Vec<Span>, IndexerApiError> {
        let envelope = self.request("Status", EmptyPayload::default()).await?;
        expect_payload::<Vec<Span>>(envelope, "status")
    }

    pub async fn variants(&self) -> Result<Vec<PalletMeta>, IndexerApiError> {
        let envelope = self.request("Variants", EmptyPayload::default()).await?;
        expect_payload::<Vec<PalletMeta>>(envelope, "variants")
    }

    pub async fn size_on_disk(&self) -> Result<u64, IndexerApiError> {
        let envelope = self.request("SizeOnDisk", EmptyPayload::default()).await?;
        expect_payload::<u64>(envelope, "sizeOnDisk")
    }

    pub async fn get_events(
        &self,
        key: Key,
        limit: Option<u16>,
        before: Option<crate::types::EventRef>,
    ) -> Result<EventsResponse, IndexerApiError> {
        let envelope = self
            .request("GetEvents", GetEventsPayload { key, limit, before })
            .await?;
        expect_payload::<EventsResponse>(envelope, "events")
    }

    pub async fn subscribe_status(&self) -> Result<StatusSubscription, IndexerApiError> {
        let (tx, rx) = mpsc::channel(32);
        let subscription_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.status_subscribers.lock().await.insert(subscription_id, tx);

        let envelope = self.request("SubscribeStatus", EmptyPayload::default()).await?;
        let _ = expect_payload::<SubscriptionStatusPayload>(envelope, "subscriptionStatus")?;

        Ok(StatusSubscription {
            client: self.clone(),
            id: subscription_id,
            receiver: rx,
        })
    }

    pub async fn unsubscribe_status(&self) -> Result<(), IndexerApiError> {
        let envelope = self.request("UnsubscribeStatus", EmptyPayload::default()).await?;
        let _ = expect_payload::<SubscriptionStatusPayload>(envelope, "subscriptionStatus")?;
        self.status_subscribers.lock().await.clear();
        Ok(())
    }

    pub async fn subscribe_events(&self, key: Key) -> Result<EventSubscription, IndexerApiError> {
        let (tx, rx) = mpsc::channel(32);
        let subscription_id = self.next_id.fetch_add(1, Ordering::Relaxed);
        self.event_subscribers.lock().await.insert(
            subscription_id,
            EventSubscriber {
                key: key.clone(),
                sender: tx,
            },
        );

        let envelope = self
            .request("SubscribeEvents", SubscribeEventsPayload { key: key.clone() })
            .await?;
        let _ = expect_payload::<SubscriptionStatusPayload>(envelope, "subscriptionStatus")?;

        Ok(EventSubscription {
            client: self.clone(),
            id: subscription_id,
            key,
            receiver: rx,
        })
    }

    pub async fn unsubscribe_events(&self, key: Key) -> Result<(), IndexerApiError> {
        let envelope = self
            .request("UnsubscribeEvents", SubscribeEventsPayload { key: key.clone() })
            .await?;
        let _ = expect_payload::<SubscriptionStatusPayload>(envelope, "subscriptionStatus")?;
        self.event_subscribers
            .lock()
            .await
            .retain(|_, subscriber| subscriber.key != key);
        Ok(())
    }

    async fn unregister_status_subscription(&self, id: u64) {
        self.status_subscribers.lock().await.remove(&id);
    }

    async fn unregister_event_subscription(&self, id: u64) {
        self.event_subscribers.lock().await.remove(&id);
    }

    async fn request<T>(&self, message_type: &'static str, payload: T) -> Result<Envelope, IndexerApiError>
    where
        T: serde::Serialize,
    {
        let id = self.next_id.fetch_add(1, Ordering::Relaxed);
        let request = RequestMessage {
            id,
            message_type,
            payload,
        };
        let json = serde_json::to_string(&request)?;
        let (tx, rx) = oneshot::channel();
        self.pending.lock().await.insert(id, tx);

        let send_result = self.writer.lock().await.send(Message::Text(json.into())).await;
        if let Err(error) = send_result {
            self.pending.lock().await.remove(&id);
            return Err(error.into());
        }

        match rx.await {
            Ok(result) => result,
            Err(_) => Err(IndexerApiError::ResponseChannelClosed { request_id: id }),
        }
    }
}

impl StatusSubscription {
    pub async fn next(&mut self) -> Option<Result<StatusUpdate, IndexerApiError>> {
        self.receiver.recv().await
    }

    pub async fn unsubscribe(self) -> Result<(), IndexerApiError> {
        let client = self.client.clone();
        let id = self.id;
        let result = client.unsubscribe_status().await;
        client.unregister_status_subscription(id).await;
        result
    }
}

impl EventSubscription {
    pub async fn next(&mut self) -> Option<Result<EventNotification, IndexerApiError>> {
        self.receiver.recv().await
    }

    pub async fn unsubscribe(self) -> Result<(), IndexerApiError> {
        let client = self.client.clone();
        let id = self.id;
        let key = self.key.clone();
        let result = client.unsubscribe_events(key).await;
        client.unregister_event_subscription(id).await;
        result
    }
}

impl Drop for StatusSubscription {
    fn drop(&mut self) {
        let client = self.client.clone();
        let id = self.id;
        tokio::spawn(async move {
            client.unregister_status_subscription(id).await;
        });
    }
}

impl Drop for EventSubscription {
    fn drop(&mut self) {
        let client = self.client.clone();
        let id = self.id;
        tokio::spawn(async move {
            client.unregister_event_subscription(id).await;
        });
    }
}

async fn run_reader(
    mut reader: futures::stream::SplitStream<tokio_tungstenite::WebSocketStream<tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>>>,
    pending: Arc<Mutex<HashMap<u64, PendingSender>>>,
    status_subscribers: StatusSubscribers,
    event_subscribers: EventSubscribers,
) {
    while let Some(message) = reader.next().await {
        match handle_message(message, &pending, &status_subscribers, &event_subscribers).await {
            Ok(()) => {}
            Err(error) => {
                fail_all_pending(&pending, &error).await;
                broadcast_status_error(&status_subscribers, &error).await;
                broadcast_event_error(&event_subscribers, &error).await;
                return;
            }
        }
    }

    let error = IndexerApiError::ConnectionClosed;
    fail_all_pending(&pending, &error).await;
    broadcast_status_error(&status_subscribers, &error).await;
    broadcast_event_error(&event_subscribers, &error).await;
}

async fn handle_message(
    message: Result<Message, tokio_tungstenite::tungstenite::Error>,
    pending: &Arc<Mutex<HashMap<u64, PendingSender>>>,
    status_subscribers: &StatusSubscribers,
    event_subscribers: &EventSubscribers,
) -> Result<(), IndexerApiError> {
    let payload = match message? {
        Message::Text(text) => text.to_string(),
        Message::Binary(bytes) => {
            String::from_utf8(bytes.to_vec()).map_err(|_| IndexerApiError::NonUtf8Binary)?
        }
        Message::Ping(_) | Message::Pong(_) | Message::Frame(_) => return Ok(()),
        Message::Close(_) => return Err(IndexerApiError::ConnectionClosed),
    };

    let envelope: Envelope = serde_json::from_str(&payload)?;

    if let Some(id) = envelope.id {
        if let Some(sender) = pending.lock().await.remove(&id) {
            let result = if envelope.message_type == "error" {
                let error = parse_server_error(&envelope)?;
                Err(error.into())
            } else {
                Ok(envelope)
            };
            let _ = sender.send(result);
            return Ok(());
        }
    }

    match envelope.message_type.as_str() {
        "status" => {
            let spans = envelope_data::<Vec<Span>>(&envelope)?;
            broadcast_status_update(status_subscribers, StatusUpdate { spans }).await;
        }
        "eventNotification" => {
            let payload = envelope_data::<EventNotificationPayload>(&envelope)?;
            broadcast_event_update(
                event_subscribers,
                EventNotification {
                    key: payload.key,
                    event: payload.event,
                    decoded_event: payload.decoded_event,
                },
            )
            .await;
        }
        "subscriptionTerminated" => {
            let termination = envelope_data::<SubscriptionTerminatedPayload>(&envelope)?;
            let subscription_error = SubscriptionTerminated {
                reason: termination.reason,
                message: termination.message,
            };
            let status_error = IndexerApiError::StatusSubscriptionTerminated {
                reason: subscription_error.reason.clone(),
                message: subscription_error.message.clone(),
            };
            let event_error = IndexerApiError::EventSubscriptionTerminated {
                reason: subscription_error.reason,
                message: subscription_error.message,
            };
            broadcast_status_error(status_subscribers, &status_error).await;
            broadcast_event_error(event_subscribers, &event_error).await;
        }
        "error" => {
            let error = parse_server_error(&envelope)?;
            let error = IndexerApiError::from(error);
            broadcast_status_error(status_subscribers, &error).await;
            broadcast_event_error(event_subscribers, &error).await;
        }
        _ => {}
    }

    Ok(())
}

fn expect_payload<T>(envelope: Envelope, expected_type: &'static str) -> Result<T, IndexerApiError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    if envelope.message_type != expected_type {
        return Err(IndexerApiError::UnexpectedResponseType {
            request_id: envelope.id.unwrap_or_default(),
            message_type: envelope.message_type,
        });
    }

    envelope_data(&envelope)
}

fn envelope_data<T>(envelope: &Envelope) -> Result<T, IndexerApiError>
where
    T: for<'de> serde::Deserialize<'de>,
{
    serde_json::from_value(
        envelope
            .data
            .clone()
            .ok_or(IndexerApiError::Json(serde_json::Error::io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                "missing data field",
            ))))?,
    )
    .map_err(IndexerApiError::from)
}

fn parse_server_error(envelope: &Envelope) -> Result<ServerError, IndexerApiError> {
    let payload = envelope_data::<ErrorPayload>(envelope)?;
    Ok(ServerError {
        code: payload.code,
        message: payload.message,
    })
}

async fn fail_all_pending(
    pending: &Arc<Mutex<HashMap<u64, PendingSender>>>,
    error: &IndexerApiError,
) {
    let mut pending = pending.lock().await;
    for (request_id, sender) in pending.drain() {
        let _ = sender.send(Err(match error {
            IndexerApiError::ConnectionClosed => IndexerApiError::RequestCancelled { request_id },
            _ => IndexerApiError::BackgroundTaskEnded,
        }));
    }
}

async fn broadcast_status_update(
    subscribers: &StatusSubscribers,
    update: StatusUpdate,
) {
    let mut subscribers = subscribers.lock().await;
    let ids: Vec<u64> = subscribers.keys().copied().collect();
    for id in ids {
        let Some(subscriber) = subscribers.get(&id).cloned() else {
            continue;
        };
        if subscriber.send(Ok(update.clone())).await.is_err() {
            subscribers.remove(&id);
        }
    }
}

async fn broadcast_event_update(
    subscribers: &EventSubscribers,
    update: EventNotification,
) {
    let mut subscribers = subscribers.lock().await;
    let ids: Vec<u64> = subscribers.keys().copied().collect();
    for id in ids {
        let Some(subscriber) = subscribers.get(&id).cloned() else {
            continue;
        };
        if subscriber.key == update.key && subscriber.sender.send(Ok(update.clone())).await.is_err() {
            subscribers.remove(&id);
        }
    }
}

async fn broadcast_status_error(
    subscribers: &StatusSubscribers,
    error: &IndexerApiError,
) {
    let mut subscribers = subscribers.lock().await;
    let ids: Vec<u64> = subscribers.keys().copied().collect();
    for id in ids {
        let Some(subscriber) = subscribers.get(&id).cloned() else {
            continue;
        };
        if subscriber.send(Err(clone_error(error))).await.is_err() {
            subscribers.remove(&id);
        }
    }
}

async fn broadcast_event_error(
    subscribers: &EventSubscribers,
    error: &IndexerApiError,
) {
    let mut subscribers = subscribers.lock().await;
    let ids: Vec<u64> = subscribers.keys().copied().collect();
    for id in ids {
        let Some(subscriber) = subscribers.get(&id).cloned() else {
            continue;
        };
        if subscriber.sender.send(Err(clone_error(error))).await.is_err() {
            subscribers.remove(&id);
        }
    }
}

    fn clone_error(error: &IndexerApiError) -> IndexerApiError {
    match error {
        IndexerApiError::Url(error) => IndexerApiError::Url(*error),
        IndexerApiError::WebSocket(_) => IndexerApiError::BackgroundTaskEnded,
        IndexerApiError::Json(error) => IndexerApiError::Json(serde_json::Error::io(std::io::Error::new(std::io::ErrorKind::InvalidData, error.to_string()))),
        IndexerApiError::RequestCancelled { request_id } => IndexerApiError::RequestCancelled { request_id: *request_id },
        IndexerApiError::ResponseChannelClosed { request_id } => IndexerApiError::ResponseChannelClosed { request_id: *request_id },
        IndexerApiError::Server { code, message } => IndexerApiError::Server { code: code.clone(), message: message.clone() },
        IndexerApiError::StatusSubscriptionTerminated { reason, message } => IndexerApiError::StatusSubscriptionTerminated { reason: reason.clone(), message: message.clone() },
        IndexerApiError::EventSubscriptionTerminated { reason, message } => IndexerApiError::EventSubscriptionTerminated { reason: reason.clone(), message: message.clone() },
        IndexerApiError::UnexpectedResponseType { request_id, message_type } => IndexerApiError::UnexpectedResponseType { request_id: *request_id, message_type: message_type.clone() },
        IndexerApiError::NonUtf8Binary => IndexerApiError::NonUtf8Binary,
        IndexerApiError::ConnectionClosed => IndexerApiError::ConnectionClosed,
        IndexerApiError::BackgroundTaskEnded => IndexerApiError::BackgroundTaskEnded,
    }
    }

    #[cfg(test)]
    mod tests {
    use super::*;
    use crate::types::{
        CustomKey, CustomScalarValue, CustomValue, DecodedEvent, Envelope, EventRef,
    };
    use serde_json::json;
    use tokio::net::TcpListener;
    use tokio::sync::mpsc;
    use tokio_tungstenite::accept_async;

    fn custom_u32_key(name: &str, value: u32) -> Key {
        Key::Custom(CustomKey {
            name: name.into(),
            value: CustomValue::U32(value),
        })
    }

    fn composite_key(name: &str, bytes: u8, value: u32) -> Key {
        Key::Custom(CustomKey {
            name: name.into(),
            value: CustomValue::Composite(vec![
                CustomScalarValue::Bytes32(crate::types::Bytes32([bytes; 32])),
                CustomScalarValue::U32(value),
            ]),
        })
    }

    #[test]
    fn parses_status_payload() {
        let envelope = Envelope {
            id: Some(2),
            message_type: "status".into(),
            data: Some(json!([{"start": 1, "end": 8}])),
        };

        let spans = expect_payload::<Vec<Span>>(envelope, "status").unwrap();
        assert_eq!(spans, vec![Span { start: 1, end: 8 }]);
    }

    #[test]
    fn expect_payload_rejects_unexpected_response_type() {
        let envelope = Envelope {
            id: Some(2),
            message_type: "variants".into(),
            data: Some(json!([])),
        };

        let error = expect_payload::<Vec<Span>>(envelope, "status").unwrap_err();
        match error {
            IndexerApiError::UnexpectedResponseType {
                request_id,
                message_type,
            } => {
                assert_eq!(request_id, 2);
                assert_eq!(message_type, "variants");
            }
            _ => panic!("unexpected error variant"),
        }
    }

    #[test]
    fn envelope_data_rejects_missing_data() {
        let envelope = Envelope {
            id: Some(2),
            message_type: "status".into(),
            data: None,
        };

        let error = envelope_data::<Vec<Span>>(&envelope).unwrap_err();
        assert!(error.to_string().contains("missing data field"));
    }

    #[test]
    fn parses_events_payload() {
        let envelope = Envelope {
            id: Some(3),
            message_type: "events".into(),
            data: Some(json!({
                "key": {"type": "Custom", "value": {"name": "ref_index", "kind": "u32", "value": 42}},
                "events": [{"blockNumber": 50, "eventIndex": 3}],
                "decodedEvents": [{
                    "blockNumber": 50,
                    "eventIndex": 3,
                    "event": {
                        "specVersion": 1234,
                        "palletName": "Referenda",
                        "eventName": "Submitted",
                        "palletIndex": 42,
                        "variantIndex": 0,
                        "eventIndex": 3,
                        "fields": {"index": 42}
                    }
                }]
            })),
        };

        let response = expect_payload::<EventsResponse>(envelope, "events").unwrap();
        assert_eq!(response.events.len(), 1);
        assert_eq!(response.decoded_events.len(), 1);
    }

    #[test]
    fn parses_server_error_payload() {
        let envelope = Envelope {
            id: Some(9),
            message_type: "error".into(),
            data: Some(json!({"code": "invalid_request", "message": "missing field `id`"})),
        };

        let error = parse_server_error(&envelope).unwrap();
        assert_eq!(error.code, "invalid_request");
        assert_eq!(error.message, "missing field `id`");
    }

    #[test]
    fn handle_message_routes_response_to_matching_pending_request() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let pending = Arc::new(Mutex::new(HashMap::new()));
            let status_subscribers = Arc::new(Mutex::new(HashMap::new()));
            let event_subscribers = Arc::new(Mutex::new(HashMap::new()));
            let (tx, rx) = oneshot::channel();
            pending.lock().await.insert(7, tx);

            handle_message(
                Ok(Message::Text(
                    serde_json::to_string(&json!({
                        "id": 7,
                        "type": "status",
                        "data": [{"start": 1, "end": 9}]
                    }))
                    .unwrap()
                    .into(),
                )),
                &pending,
                &status_subscribers,
                &event_subscribers,
            )
            .await
            .unwrap();

            let response = rx.await.unwrap().unwrap();
            assert_eq!(response.id, Some(7));
            assert_eq!(response.message_type, "status");
        });
    }

    #[test]
    fn handle_message_routes_server_error_to_matching_pending_request() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let pending = Arc::new(Mutex::new(HashMap::new()));
            let status_subscribers = Arc::new(Mutex::new(HashMap::new()));
            let event_subscribers = Arc::new(Mutex::new(HashMap::new()));
            let (tx, rx) = oneshot::channel();
            pending.lock().await.insert(9, tx);

            handle_message(
                Ok(Message::Text(
                    serde_json::to_string(&json!({
                        "id": 9,
                        "type": "error",
                        "data": {"code": "invalid_request", "message": "missing field `id`"}
                    }))
                    .unwrap()
                    .into(),
                )),
                &pending,
                &status_subscribers,
                &event_subscribers,
            )
            .await
            .unwrap();

            let error = rx.await.unwrap().unwrap_err();
            match error {
                IndexerApiError::Server { code, message } => {
                    assert_eq!(code, "invalid_request");
                    assert_eq!(message, "missing field `id`");
                }
                _ => panic!("unexpected error variant"),
            }
        });
    }

    #[test]
    fn handle_message_broadcasts_status_update_to_subscribers() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let pending = Arc::new(Mutex::new(HashMap::new()));
            let status_subscribers = Arc::new(Mutex::new(HashMap::new()));
            let event_subscribers = Arc::new(Mutex::new(HashMap::new()));
            let (tx, mut rx) = mpsc::channel(1);
            status_subscribers.lock().await.insert(1, tx);

            handle_message(
                Ok(Message::Text(
                    serde_json::to_string(&json!({
                        "type": "status",
                        "data": [{"start": 1, "end": 8}]
                    }))
                    .unwrap()
                    .into(),
                )),
                &pending,
                &status_subscribers,
                &event_subscribers,
            )
            .await
            .unwrap();

            let update = rx.recv().await.unwrap().unwrap();
            assert_eq!(update, StatusUpdate { spans: vec![Span { start: 1, end: 8 }] });
        });
    }

    #[test]
    fn handle_message_broadcasts_event_notification_to_subscribers() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let pending = Arc::new(Mutex::new(HashMap::new()));
            let status_subscribers = Arc::new(Mutex::new(HashMap::new()));
            let event_subscribers = Arc::new(Mutex::new(HashMap::new()));
            let (tx, mut rx) = mpsc::channel(1);
            event_subscribers.lock().await.insert(1, EventSubscriber { key: custom_u32_key("ref_index", 42), sender: tx });

            handle_message(
                Ok(Message::Text(
                    serde_json::to_string(&json!({
                        "type": "eventNotification",
                        "data": {
                            "key": {"type": "Custom", "value": {"name": "ref_index", "kind": "u32", "value": 42}},
                            "event": {"blockNumber": 50, "eventIndex": 3},
                            "decodedEvent": null
                        }
                    }))
                    .unwrap()
                    .into(),
                )),
                &pending,
                &status_subscribers,
                &event_subscribers,
            )
            .await
            .unwrap();

            let update = rx.recv().await.unwrap().unwrap();
            assert_eq!(update.key, custom_u32_key("ref_index", 42));
            assert_eq!(update.event, EventRef { block_number: 50, event_index: 3 });
            assert!(update.decoded_event.is_none());
        });
    }

    #[test]
    fn handle_message_broadcasts_subscription_termination_to_subscribers() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let pending = Arc::new(Mutex::new(HashMap::new()));
            let status_subscribers = Arc::new(Mutex::new(HashMap::new()));
            let event_subscribers = Arc::new(Mutex::new(HashMap::new()));
            let (status_tx, mut status_rx) = mpsc::channel(1);
            let (event_tx, mut event_rx) = mpsc::channel(1);
            status_subscribers.lock().await.insert(1, status_tx);
            event_subscribers.lock().await.insert(2, EventSubscriber { key: custom_u32_key("ref_index", 42), sender: event_tx });

            handle_message(
                Ok(Message::Text(
                    serde_json::to_string(&json!({
                        "type": "subscriptionTerminated",
                        "data": {
                            "reason": "backpressure",
                            "message": "subscriber disconnected due to backpressure"
                        }
                    }))
                    .unwrap()
                    .into(),
                )),
                &pending,
                &status_subscribers,
                &event_subscribers,
            )
            .await
            .unwrap();

            match status_rx.recv().await.unwrap().unwrap_err() {
                IndexerApiError::StatusSubscriptionTerminated { reason, message } => {
                    assert_eq!(reason, "backpressure");
                    assert_eq!(message, "subscriber disconnected due to backpressure");
                }
                _ => panic!("unexpected status error variant"),
            }

            match event_rx.recv().await.unwrap().unwrap_err() {
                IndexerApiError::EventSubscriptionTerminated { reason, message } => {
                    assert_eq!(reason, "backpressure");
                    assert_eq!(message, "subscriber disconnected due to backpressure");
                }
                _ => panic!("unexpected event error variant"),
            }
        });
    }

    #[test]
    fn handle_message_rejects_invalid_binary_payload() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let pending = Arc::new(Mutex::new(HashMap::new()));
            let status_subscribers = Arc::new(Mutex::new(HashMap::new()));
            let event_subscribers = Arc::new(Mutex::new(HashMap::new()));

            let error = handle_message(
                Ok(Message::Binary(vec![0xFF, 0xFE].into())),
                &pending,
                &status_subscribers,
                &event_subscribers,
            )
            .await
            .unwrap_err();

            assert!(matches!(error, IndexerApiError::NonUtf8Binary));
        });
    }

    #[test]
    fn fail_all_pending_marks_connection_closed_requests_as_cancelled() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let pending = Arc::new(Mutex::new(HashMap::new()));
            let (tx, rx) = oneshot::channel();
            pending.lock().await.insert(12, tx);

            fail_all_pending(&pending, &IndexerApiError::ConnectionClosed).await;

            match rx.await.unwrap().unwrap_err() {
                IndexerApiError::RequestCancelled { request_id } => assert_eq!(request_id, 12),
                _ => panic!("unexpected error variant"),
            }
        });
    }

    #[test]
    fn fail_all_pending_marks_other_failures_as_background_task_ended() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let pending = Arc::new(Mutex::new(HashMap::new()));
            let (tx, rx) = oneshot::channel();
            pending.lock().await.insert(13, tx);

            fail_all_pending(
                &pending,
                &IndexerApiError::Server {
                    code: "internal_error".into(),
                    message: "boom".into(),
                },
            )
            .await;

            assert!(matches!(
                rx.await.unwrap().unwrap_err(),
                IndexerApiError::BackgroundTaskEnded
            ));
        });
    }

    #[test]
    fn clone_error_preserves_server_payload() {
        let cloned = clone_error(&IndexerApiError::Server {
            code: "invalid_request".into(),
            message: "missing field `id`".into(),
        });

        match cloned {
            IndexerApiError::Server { code, message } => {
                assert_eq!(code, "invalid_request");
                assert_eq!(message, "missing field `id`");
            }
            _ => panic!("unexpected error variant"),
        }
    }

    #[test]
    fn event_notification_payload_matches_server_shape() {
        let payload = serde_json::from_value::<EventNotificationPayload>(json!({
            "key": {"type": "Custom", "value": {"name": "item_id", "kind": "bytes32", "value": format!("0x{}", "11".repeat(32))}},
            "event": {"blockNumber": 50, "eventIndex": 3},
            "decodedEvent": {
                "blockNumber": 50,
                "eventIndex": 3,
                "event": {
                    "specVersion": 1234,
                    "palletName": "Content",
                    "eventName": "PublishRevision",
                    "palletIndex": 42,
                    "variantIndex": 1,
                    "eventIndex": 3,
                    "fields": {}
                }
            }
        }))
        .unwrap();

        assert_eq!(payload.event, EventRef { block_number: 50, event_index: 3 });
        assert_eq!(payload.key, Key::Custom(CustomKey { name: "item_id".into(), value: CustomValue::Bytes32(crate::types::Bytes32([0x11; 32])) }));
        assert_eq!(
            payload.decoded_event,
            Some(DecodedEvent {
                block_number: 50,
                event_index: 3,
                event: crate::types::StoredEvent {
                    spec_version: 1234,
                    pallet_name: "Content".into(),
                    event_name: "PublishRevision".into(),
                    pallet_index: 42,
                    variant_index: 1,
                    event_index: 3,
                    fields: json!({}),
                },
            })
        );
    }

    #[test]
    fn broadcast_event_update_only_notifies_matching_keys() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let subscribers = Arc::new(Mutex::new(HashMap::new()));
            let (match_tx, mut match_rx) = mpsc::channel(1);
            let (other_tx, mut other_rx) = mpsc::channel(1);

            subscribers.lock().await.insert(
                1,
                EventSubscriber {
                    key: custom_u32_key("ref_index", 42),
                    sender: match_tx,
                },
            );
            subscribers.lock().await.insert(
                2,
                EventSubscriber {
                    key: custom_u32_key("ref_index", 7),
                    sender: other_tx,
                },
            );

            broadcast_event_update(
                &subscribers,
                EventNotification {
                    key: custom_u32_key("ref_index", 42),
                    event: EventRef {
                        block_number: 10,
                        event_index: 1,
                    },
                    decoded_event: None,
                },
            )
            .await;

            assert!(match_rx.recv().await.is_some());
            assert!(other_rx.try_recv().is_err());
        });
    }

    #[test]
    fn broadcast_event_update_matches_composite_keys() {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        runtime.block_on(async {
            let subscribers = Arc::new(Mutex::new(HashMap::new()));
            let (match_tx, mut match_rx) = mpsc::channel(1);
            let (other_tx, mut other_rx) = mpsc::channel(1);

            subscribers.lock().await.insert(
                1,
                EventSubscriber {
                    key: composite_key("item_revision", 0x11, 7),
                    sender: match_tx,
                },
            );
            subscribers.lock().await.insert(
                2,
                EventSubscriber {
                    key: composite_key("item_revision", 0x11, 8),
                    sender: other_tx,
                },
            );

            broadcast_event_update(
                &subscribers,
                EventNotification {
                    key: composite_key("item_revision", 0x11, 7),
                    event: EventRef {
                        block_number: 10,
                        event_index: 1,
                    },
                    decoded_event: None,
                },
            )
            .await;

            assert!(match_rx.recv().await.is_some());
            assert!(other_rx.try_recv().is_err());
        });
    }

    #[test]
    fn clone_error_preserves_response_channel_closed_payload() {
        let cloned = clone_error(&IndexerApiError::ResponseChannelClosed { request_id: 44 });

        match cloned {
            IndexerApiError::ResponseChannelClosed { request_id } => assert_eq!(request_id, 44),
            _ => panic!("unexpected error variant"),
        }
    }

    #[tokio::test(flavor = "current_thread")]
    async fn close_sends_websocket_close_frame() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut websocket = accept_async(stream).await.unwrap();

            match websocket.next().await {
                Some(Ok(Message::Close(_))) => {}
                other => panic!("expected websocket close frame, got {other:?}"),
            }
        });

        let client = IndexerClient::connect(&format!("ws://{addr}")).await.unwrap();
        client.close().await.unwrap();

        server.await.unwrap();
    }
}
