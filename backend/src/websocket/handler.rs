

use crate::websocket::types::WebSocketMessage;
use axum::extract::ws::{Message, WebSocket};
use tokio::sync::{broadcast, Mutex};
use serde_json::Value;
use std::sync::Arc;


pub struct WebSocketHandler {
    socket: WebSocket,
    doc_id: String,
    tx: broadcast::Sender<WebSocketMessage>
}

impl WebSocketHandler {
    pub fn new(socket: WebSocket, doc_id: String, tx: broadcast::Sender<WebSocketMessage>) -> Self {
        Self { socket, doc_id, tx }
    }

    pub async fn handle(&mut self) {
        let mut rx = self.tx.subscribe();

        loop {
            tokio::select! {
                msg = self.socket.recv() => {
                    if !self.handle_message(msg).await {
                        break;
                    }
                }
                Ok(update) = rx.recv() => {
                    if let Ok(msg) = serde_json::to_string(&update) {
                        if self.socket.send(Message::Text(msg)).await.is_err() {
                            break;
                        }
                    }
                }
            }
        }
    }

    async fn handle_message(&mut self, msg: Option<Result<Message, axum::Error>>) -> bool {
        match msg {
            Some(Ok(Message::Text(text))) => {
                let message: WebSocketMessage = match serde_json::from_str(&text) {
                    Ok(msg) => msg,
                    Err(e) => {
                        eprintln!("Failed to deserialize WebSocket message: {}", e);
                        return true;
                    }
                };

                match message.type_.as_str() {
                    "init" => {
                        println!("Initializing document: {}", message.doc_id);
                        if let Err(e) = message.content.doc.validate() {
                            eprintln!("Invalid document structure: {}", e);
                        }
                    }
                    "update" => {
                        if let Some(steps) = message.steps {
                            println!("Processing steps for document: {}", message.doc_id);
                            for step in steps {
                                println!("Step: {:?}", step);
                            }
                        }
                    }
                    _ => eprintln!("Unknown message type: {}", message.type_),
                }
                true
            }
            None | Some(Err(_)) => false,
            _ => true,
        }
    }
}