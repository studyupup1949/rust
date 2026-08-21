use futures::channel::mpsc::{unbounded, UnboundedSender};
use futures::{SinkExt, StreamExt};

use tokio_tungstenite::tungstenite::Message;

use log::{debug, error, info, warn};

use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, RwLock};

pub mod config;
pub mod error;
mod local;

use crate::config::*;
use crate::error::*;
use actnel_lib::*;

use std::time::Duration;
use tokio::sync::{mpsc, Mutex};

pub type ActiveStreams = Arc<RwLock<HashMap<StreamId, UnboundedSender<StreamMessage>>>>;

lazy_static::lazy_static! {
    pub static ref ACTIVE_STREAMS:ActiveStreams = Arc::new(RwLock::new(HashMap::new()));
    pub static ref RECONNECT_TOKEN: Arc<Mutex<Option<ReconnectToken>>> = Arc::new(Mutex::new(None));
}

#[derive(Debug, Clone)]
pub enum StreamMessage {
    Data(Vec<u8>),
    Close,
}

pub struct Session {
    config: Config,
    wormhole: Wormhole,
}

impl Session {
    pub async fn connect(config: Config) -> Result<Self, Error> {
        // let config = match Config::get() {
        //     Ok(config) => config,
        //     Err(_) => return Err(Error::Timeout),
        // };

        let wormhole = Wormhole::connect(&config).await?;
        Ok(Session {
            config,
            wormhole,
        })
    }

    pub async fn listen(&self) -> Result<SocketAddr, Error> {
        let config = self.config.clone();
        let (restart_tx, _) = unbounded();
        let _ = self.wormhole.listen(config, restart_tx).await;
        // self.config.first_run = false;
        Ok(self.config.local_addr.clone())
    }

    pub async fn close(&self) -> Result<(), Error> {
        let _ = self.wormhole.close().await;
        Ok(())
    }

    pub fn ingress_url(&self) -> String {
        self.config.activation_url(self.wormhole.hostname.as_str())
    }
}

struct Wormhole {
    sender: mpsc::UnboundedSender<Message>,
    receiver: Arc<Mutex<mpsc::UnboundedReceiver<Message>>>,
    sub_domain: String,
    hostname: String,
}

impl Wormhole {
    // Function to create a new Wormhole connection
    async fn connect(config: &Config) -> Result<Self, Error> {
        let (mut websocket, _) = tokio_tungstenite::connect_async(&config.control_url).await?;

        // send our Client Hello message
        let client_hello = match config.secret_key.clone() {
            Some(secret_key) => ClientHello::generate(
                config.sub_domain.clone(),
                ClientType::Auth { key: secret_key },
            ),
            None => {
                // if we have a reconnect token, use it.
                if let Some(reconnect) = RECONNECT_TOKEN.lock().await.clone() {
                    ClientHello::reconnect(reconnect)
                } else {
                    ClientHello::generate(config.sub_domain.clone(), ClientType::Anonymous)
                }
            }
        };

        info!("connecting to wormhole...");

        let hello = serde_json::to_vec(&client_hello).unwrap();
        websocket
            .send(Message::binary(hello))
            .await
            .expect("Failed to send client hello to wormhole server.");

        // wait for Server hello
        let server_hello_data = websocket
            .next()
            .await
            .ok_or(Error::NoResponseFromServer)??
            .into_data();
        let server_hello = serde_json::from_slice::<ServerHello>(&server_hello_data).map_err(|e| {
            error!("Couldn't parse server_hello from {:?}", e);
            Error::ServerReplyInvalid
        })?;

        let (sub_domain, hostname) = match server_hello {
            ServerHello::Success {
                sub_domain,
                client_id,
                hostname,
            } => {
                info!("Server accepted our connection. I am client_{}", client_id);
                (sub_domain, hostname)
            }
            ServerHello::AuthFailed => {
                return Err(Error::AuthenticationFailed);
            }
            ServerHello::InvalidSubDomain => {
                return Err(Error::InvalidSubDomain);
            }
            ServerHello::SubDomainInUse => {
                return Err(Error::SubDomainInUse);
            }
            ServerHello::Error(error) => return Err(Error::ServerError(error)),
        };

        let (receive_tx, receive_rx) = mpsc::unbounded_channel();
        let (send_tx, mut send_rx) = mpsc::unbounded_channel();
        // Spawn a task to handle the WebSocket
        tokio::spawn({
            async move {
                let mut ws_stream = websocket;
                loop {
                    tokio::select! {
                        message = ws_stream.next() => {
                            match message {
                                Some(Ok(msg)) => {
                                    if receive_tx.send(msg).is_err() {
                                        break; // Channel closed
                                    }
                                }
                                Some(Err(e)) => { // WebSocket error
                                    warn!("websocket read error: {:?}", e);
                                    break;
                                },
                                None => { // WebSocket closed
                                    warn!("websocket sent none");
                                    break;
                                },
                            }
                        }
                        received = async {
                            send_rx.recv().await
                        } => {
                            // received is the result of locked_receiver.recv().await
                            if let Some(msg) = received {
                                if ws_stream.send(msg).await.is_err() {
                                    break; // WebSocket error or closed
                                }
                            } else {
                                break; // Channel closed
                            }
                        }
                    }
                }
            }
        });

        Ok(Wormhole {
            sender: send_tx,
            receiver: Arc::new(Mutex::new(receive_rx)),
            sub_domain,
            hostname,
        })
    }

    // Send a message through the WebSocket
    pub async fn send_message(&self, message: Message) -> Result<(), mpsc::error::SendError<Message>> {
        self.sender.send(message)
    }

    // Receive a message from the WebSocket
    pub async fn receive_message(&self) -> Option<Message> {
        self.receiver.lock().await.recv().await
    }

    // Close the WebSocket
    pub async fn close(&self) -> Result<(), mpsc::error::SendError<Message>> {
        // Set the reconnect token to None
        let _ = RECONNECT_TOKEN.lock().await.take();
        // Send a close message
        self.sender.send(Message::Close(None))
        // solution for Protocol(ResetWithoutClosingHandshake)
        // but doesn't work
        // let _ = self.sender.send(Message::Close(None));
        // let mut non_close_message_count = 0;
        // while let Some(message) = self.receiver.lock().await.recv().await {
        //     match message {
        //         Message::Close(_) => {
        //             // Acknowledgment received, break the loop
        //             break;
        //         }
        //         _ => {
        //             // Log other messages if necessary, but keep waiting for the close acknowledgment
        //             debug!("Received non-close message during closing process: {:?}", message);
        //             non_close_message_count += 1;
        //             if non_close_message_count >= 10 {
        //                 // Error out for timeout
        //                 break;
        //             }
        //         }
        //     }
        // }
        // Ok(())
    }

    async fn listen(&self, config: Config, restart_tx: UnboundedSender<Option<Error>>) -> Result<(), Error> {
        // tunnel channel
        let (tunnel_tx, mut tunnel_rx) = unbounded::<ControlPacket>();

        // continuously write to websocket tunnel
        let mut restart = restart_tx.clone();
        let sender_clone = self.sender.clone();
        tokio::spawn(async move {
            while let Some(packet) = tunnel_rx.next().await {
                let message = Message::binary(packet.serialize());  // Assuming ControlPacket has a serialize method
                match sender_clone.send(message) {
                    Ok(_) => {}  // Successfully sent the message
                    Err(e) => {
                        // Handle the error
                        warn!("Failed to send message to WebSocket tunnel: {:?}", e);
                        // let _ = restart_tx.send(Some(Error::WebSocketError(e))).await;
                        let _ = restart.send(Some(Error::Timeout)).await;
                        return;
                    }
                }
            }
        });

        // continuously read from websocket tunnel
        let mut restart = restart_tx.clone();
        let receiver_clone = self.receiver.clone();
        tokio::spawn(async move {
            loop {
                let mut receiver = receiver_clone.lock().await;
                match receiver.recv().await {
                    Some(message) if message.is_close() => {
                        debug!("got close message");
                        let _ = restart.send(None).await;
                        return Ok(());
                    }
                    Some(message) => {
                        let packet = process_control_flow_message(
                            config.clone(),
                            tunnel_tx.clone(),
                            message.into_data(),
                        )
                        .await
                        .map_err(|e| {
                            error!("Malformed protocol control packet: {:?}", e);
                            Error::MalformedMessageFromServer
                        })?;
                        debug!("Processed packet: {:?}", packet.packet_type());
                    }
                    None => {
                        warn!("websocket sent none");
                        return Err(Error::Timeout);
                    }
                }
            }
        });

        Ok(())
    }
}

async fn process_control_flow_message(
    config: Config,
    mut tunnel_tx: UnboundedSender<ControlPacket>,
    payload: Vec<u8>,
) -> Result<ControlPacket, Box<dyn std::error::Error>> {
    let control_packet = ControlPacket::deserialize(&payload)?;

    match &control_packet {
        ControlPacket::Init(stream_id) => {
            info!("stream[{:?}] -> init", stream_id.to_string());
        }
        ControlPacket::Ping(reconnect_token) => {
            log::info!("got ping. reconnect_token={}", reconnect_token.is_some());

            if let Some(reconnect) = reconnect_token {
                let _ = RECONNECT_TOKEN.lock().await.replace(reconnect.clone());
            }
            let _ = tunnel_tx.send(ControlPacket::Ping(None)).await;
        }
        ControlPacket::Refused(_) => return Err("unexpected control packet".into()),
        ControlPacket::End(stream_id) => {
            // find the stream
            let stream_id = stream_id.clone();

            info!("got end stream [{:?}]", &stream_id);

            tokio::spawn(async move {
                let stream = ACTIVE_STREAMS.read().unwrap().get(&stream_id).cloned();
                if let Some(mut tx) = stream {
                    tokio::time::sleep(Duration::from_secs(5)).await;
                    let _ = tx.send(StreamMessage::Close).await.map_err(|e| {
                        error!("failed to send stream close: {:?}", e);
                    });
                    ACTIVE_STREAMS.write().unwrap().remove(&stream_id);
                }
            });
        }
        ControlPacket::Data(stream_id, data) => {
            info!(
                "stream[{:?}] -> new data: {:?}",
                stream_id.to_string(),
                data.len()
            );

            if !ACTIVE_STREAMS.read().unwrap().contains_key(&stream_id) {
                if local::setup_new_stream(config.clone(), tunnel_tx.clone(), stream_id.clone())
                    .await
                    .is_none()
                {
                    error!("failed to open local tunnel")
                }
            }

            // find the right stream
            let active_stream = ACTIVE_STREAMS.read().unwrap().get(&stream_id).cloned();

            // forward data to it
            if let Some(mut tx) = active_stream {
                tx.send(StreamMessage::Data(data.clone())).await?;
                info!("forwarded to local tcp ({})", stream_id.to_string());
            } else {
                error!("got data but no stream to send it to.");
                let _ = tunnel_tx
                    .send(ControlPacket::Refused(stream_id.clone()))
                    .await?;
            }
        }
    };

    Ok(control_packet.clone())
}
