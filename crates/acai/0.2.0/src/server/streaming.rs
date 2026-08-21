use std::pin::Pin;
use std::task::{Context, Poll};

use bytes::Bytes;
use hyper::body::Frame;
use tokio::sync::broadcast;

/// A Body implementation that streams Server-Sent Events (SSE) from a broadcast channel.
///
/// This allows the server to send real-time updates to connected clients using
/// the broadcast channel as an event source. The implementation handles backpressure
/// by registering for wakeups when no messages are available.
///
/// # Example
///
/// ```
/// use tokio::sync::broadcast;
/// use acai::server::streaming::SseBody;
/// use hyper::Response;
/// use http_body_util::{BodyExt, BodyStream, combinators::BoxBody};
/// use bytes::Bytes;
/// use tower::BoxError;
///
/// // Create a broadcast channel for real-time updates
/// let (sender, _) = broadcast::channel::<String>(100);
///
/// // Create an SSE body
/// let body = SseBody::from_sender(&sender);
///
/// // Use it in an HTTP response
/// let boxed_body = BodyStream::new(body).boxed();
/// let response = Response::builder()
///     .header("Content-Type", "text/event-stream")
///     .header("Cache-Control", "no-cache")
///     .header("Connection", "keep-alive")
///     .body(boxed_body)
///     .unwrap();
///
/// // Later, send updates through the channel
/// sender.send("{\"type\":\"update\",\"data\":\"new information\"}".to_string()).unwrap();
/// ```
pub struct SseBody {
    /// The broadcast receiver for incoming messages
    receiver: broadcast::Receiver<String>,
}

impl SseBody {
    /// Create a new SSE body from a broadcast receiver
    ///
    /// This constructor directly takes a broadcast receiver and wraps it in an SSE body.
    ///
    /// # Example
    ///
    /// ```
    /// use tokio::sync::broadcast;
    /// use acai::server::streaming::SseBody;
    ///
    /// let (sender, receiver) = broadcast::channel::<String>(100);
    /// let body = SseBody::new(receiver);
    /// ```
    pub fn new(receiver: broadcast::Receiver<String>) -> Self {
        Self { receiver }
    }

    /// Create a new SSE body by subscribing to a broadcast sender
    ///
    /// This constructor subscribes to the given broadcast sender and creates an SSE body
    /// from the resulting receiver. This is generally the preferred way to create an
    /// SSE body since it automatically handles the subscription.
    ///
    /// # Example
    ///
    /// ```
    /// use tokio::sync::broadcast;
    /// use acai::server::streaming::SseBody;
    ///
    /// let (sender, _) = broadcast::channel::<String>(100);
    /// let body = SseBody::from_sender(&sender);
    /// ```
    pub fn from_sender(sender: &broadcast::Sender<String>) -> Self {
        let receiver = sender.subscribe();
        Self::new(receiver)
    }
}

impl hyper::body::Body for SseBody {
    type Data = Bytes;
    type Error = Box<dyn std::error::Error + Send + Sync>;

    /// Poll for the next frame from the SSE body
    ///
    /// This method is called by hyper to get the next chunk of data. It handles:
    /// - Returning formatted SSE events when messages are available
    /// - Registering wake-ups for backpressure when the channel is empty
    /// - Ending the stream when the channel is closed
    /// - Skipping lagged messages if the consumer falls behind
    fn poll_frame(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
    ) -> Poll<Option<Result<Frame<Self::Data>, Self::Error>>> {
        // Poll the broadcast receiver for new messages
        let receiver = &mut self.receiver;

        // Try to receive a message without blocking
        match receiver.try_recv() {
            Ok(message) => {
                // Format the message as an SSE event
                let formatted = format!("data: {}\n\n", message);
                Poll::Ready(Some(Ok(Frame::data(Bytes::from(formatted)))))
            }
            Err(broadcast::error::TryRecvError::Empty) => {
                // No message available, register for wakeup
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Err(broadcast::error::TryRecvError::Closed) => {
                // Channel is closed, end the stream
                Poll::Ready(None)
            }
            Err(broadcast::error::TryRecvError::Lagged(_)) => {
                // If we lag behind, just continue silently
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        }
    }

    /// Indicates whether the stream has ended
    ///
    /// Always returns false for SSE bodies since they are potentially infinite.
    /// The stream only ends when the channel is closed, which is detected in `poll_frame`.
    fn is_end_stream(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use http_body_util::BodyExt;
    use tokio::sync::broadcast;

    #[test]
    fn sse_body_from_sender() {
        // This test requires tokio runtime so we use a runtime block
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let (sender, _) = broadcast::channel::<String>(16);

            // Create the body from the sender
            let mut body = SseBody::from_sender(&sender);

            // Send a message through the channel
            sender
                .send(r#"{"type":"update","data":"test"}"#.to_string())
                .unwrap();

            // Use hyper's BodyExt to get the first frame
            let frame = body.frame().await.unwrap().unwrap();

            // Verify the frame contains the expected SSE formatted data
            assert_eq!(
                frame.into_data().unwrap().to_vec(),
                br#"data: {"type":"update","data":"test"}

"#
            );
        });
    }
}
